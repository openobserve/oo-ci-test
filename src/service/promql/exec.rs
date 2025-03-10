// Copyright 2023 Zinc Labs Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use ahash::AHashMap as HashMap;
use datafusion::error::Result;
use promql_parser::parser::EvalStmt;
use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::sync::RwLock;

use crate::common::meta::stream::ScanStats;
use crate::service::promql::{
    micros, micros_since_epoch, value::*, TableProvider, DEFAULT_LOOKBACK,
};

#[derive(Clone)]
pub struct Query {
    pub org_id: String,
    pub table_provider: Arc<Box<dyn TableProvider>>,
    /// The time boundaries for the evaluation. If start equals end an instant
    /// is evaluated.
    pub start: i64,
    pub end: i64,
    /// Time between two evaluated instants for the range [start:end].
    pub interval: i64,
    /// Default look back from sample search.
    pub lookback_delta: i64,
    /// key — metric name; value — time series data
    pub data_cache: Arc<RwLock<HashMap<String, Value>>>,
    pub scan_stats: Arc<RwLock<ScanStats>>,
}

impl Query {
    pub fn new<P>(org_id: &str, provider: P) -> Self
    where
        P: TableProvider,
    {
        let now = micros_since_epoch(SystemTime::now());
        let five_min = micros(DEFAULT_LOOKBACK);
        Self {
            org_id: org_id.to_string(),
            table_provider: Arc::new(Box::new(provider)),
            start: now,
            end: now,
            interval: five_min,
            lookback_delta: five_min,
            data_cache: Arc::new(RwLock::new(HashMap::default())),
            scan_stats: Arc::new(RwLock::new(ScanStats::default())),
        }
    }

    #[tracing::instrument(name = "promql:engine:exec", skip_all)]
    pub async fn exec(&mut self, stmt: EvalStmt) -> Result<(Value, Option<String>, ScanStats)> {
        self.start = micros_since_epoch(stmt.start);
        self.end = micros_since_epoch(stmt.end);
        if stmt.interval > Duration::ZERO {
            self.interval = micros(stmt.interval);
        }
        if stmt.lookback_delta > Duration::ZERO {
            self.lookback_delta = micros(stmt.lookback_delta);
        }

        let ctx = Arc::new(self.clone());
        let expr = Arc::new(stmt.expr);
        let mut result_type: Option<String> = None;

        // range query always be matrix result type.
        if self.start != self.end {
            result_type = Some("matrix".to_string());
        } else {
            // Instant query
            let mut engine = super::Engine::new(ctx.clone(), self.start);
            let expr = expr.clone();
            let (mut value, result_type_exec) = engine.exec(&expr).await?;
            if let Value::Float(val) = value {
                value = Value::Sample(Sample::new(self.end, val));
            }
            value.sort();
            if result_type_exec.is_some() {
                result_type = result_type_exec;
            }
            return Ok((value, result_type, *self.scan_stats.read().await));
        }

        // Range query
        // See https://promlabs.com/blog/2020/06/18/the-anatomy-of-a-promql-query/#range-queries
        let mut instant_vectors = Vec::new();
        let mut string_literals = Vec::new();
        let mut tasks = Vec::new();
        let nr_steps = ((self.end - self.start) / self.interval) + 1;
        for i in 0..nr_steps {
            let time = self.start + (self.interval * i);
            let mut engine = super::Engine::new(ctx.clone(), time);
            let expr = expr.clone();
            // let task = tokio::task::spawn(async move { (time, engine.exec(&expr).await) });
            let task = (time, engine.exec(&expr).await);
            tasks.push(task);
        }

        for task in tasks {
            let (time, result) = task;
            let (result, result_type_exec) = result?;
            if result_type.is_none() && result_type_exec.is_some() {
                result_type = result_type_exec;
            }
            match result {
                Value::Instant(v) => {
                    instant_vectors.push(RangeValue::new(v.labels.to_owned(), [v.sample]))
                }
                Value::Vector(vs) => instant_vectors.extend(
                    vs.into_iter()
                        .map(|v| RangeValue::new(v.labels.to_owned(), [v.sample])),
                ),
                Value::Range(v) => instant_vectors.push(v),
                Value::Matrix(vs) => instant_vectors.extend(vs),
                Value::Sample(s) => instant_vectors.push(RangeValue::new(Labels::default(), [s])),
                Value::Float(val) => instant_vectors
                    .push(RangeValue::new(Labels::default(), [Sample::new(time, val)])),
                Value::String(val) => string_literals.push(val),
                Value::None => continue,
            };
        }

        if !string_literals.is_empty() {
            let output_str = string_literals.join(", ");
            return Ok((
                Value::String(output_str),
                result_type,
                *self.scan_stats.read().await,
            ));
        }

        // empty result quick return
        if instant_vectors.is_empty() {
            return Ok((Value::None, result_type, *self.scan_stats.read().await));
        }

        // merge data
        let mut merged_data = HashMap::default();
        let mut merged_metrics = HashMap::default();
        for value in instant_vectors {
            merged_data
                .entry(signature(&value.labels))
                .or_insert_with(Vec::new)
                .extend(value.samples);
            merged_metrics.insert(signature(&value.labels), value.labels);
        }
        let merged_data = merged_data
            .into_iter()
            .map(|(sig, samples)| {
                RangeValue::new(merged_metrics.get(&sig).unwrap().to_owned(), samples)
            })
            .collect::<Vec<_>>();

        // sort data
        let mut value = Value::Matrix(merged_data);
        value.sort();
        Ok((value, result_type, *self.scan_stats.read().await))
    }
}
