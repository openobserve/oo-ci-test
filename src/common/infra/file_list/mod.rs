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

use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::common::{
    infra::{
        config::CONFIG,
        errors::{Error, Result},
    },
    meta::{
        common::{FileKey, FileMeta},
        stream::{PartitionTimeLevel, StreamStats},
        StreamType,
    },
};

pub mod dynamo;
pub mod postgres;
pub mod sqlite;

static CLIENT: Lazy<Box<dyn FileList>> = Lazy::new(connect);

pub fn connect() -> Box<dyn FileList> {
    match CONFIG.common.meta_store.as_str() {
        "sqlite" => Box::<sqlite::SqliteFileList>::default(),
        "postgres" | "postgresql" => Box::<postgres::PostgresFileList>::default(),
        "dynamo" | "dynamodb" => Box::<dynamo::DynamoFileList>::default(),
        _ => Box::<sqlite::SqliteFileList>::default(),
    }
}

#[async_trait]
pub trait FileList: Sync + Send + 'static {
    async fn add(&self, file: &str, meta: &FileMeta) -> Result<()>;
    async fn remove(&self, file: &str) -> Result<()>;
    async fn batch_add(&self, files: &[FileKey]) -> Result<()>;
    async fn batch_remove(&self, files: &[String]) -> Result<()>;
    async fn get(&self, file: &str) -> Result<FileMeta>;
    async fn contains(&self, file: &str) -> Result<bool>;
    async fn list(&self) -> Result<Vec<(String, FileMeta)>>;
    async fn query(
        &self,
        org_id: &str,
        stream_type: StreamType,
        stream_name: &str,
        time_level: PartitionTimeLevel,
        time_range: (i64, i64),
    ) -> Result<Vec<(String, FileMeta)>>;
    async fn get_max_pk_value(&self) -> Result<i64>;
    async fn stats(
        &self,
        org_id: &str,
        stream_type: Option<StreamType>,
        stream_name: Option<&str>,
        pk_value: Option<(i64, i64)>,
    ) -> Result<Vec<(String, StreamStats)>>;
    async fn get_stream_stats(
        &self,
        org_id: &str,
        stream_type: Option<StreamType>,
        stream_name: Option<&str>,
    ) -> Result<Vec<(String, StreamStats)>>;
    async fn set_stream_stats(&self, org_id: &str, streams: &[(String, StreamStats)])
        -> Result<()>;
    async fn reset_stream_stats_min_ts(
        &self,
        org_id: &str,
        stream: &str,
        min_ts: i64,
    ) -> Result<()>;
    async fn len(&self) -> usize;
    async fn is_empty(&self) -> bool;
    async fn clear(&self) -> Result<()>;
}

pub async fn create_table() -> Result<()> {
    // check cache dir
    std::fs::create_dir_all(&CONFIG.common.data_db_dir)?;
    match CONFIG.common.meta_store.as_str() {
        "sqlite" => sqlite::create_table().await,
        "postgres" | "postgresql" => postgres::create_table().await,
        "dynamo" | "dynamodb" => dynamo::create_table().await,
        _ => sqlite::create_table().await,
    }
}

pub async fn create_table_index() -> Result<()> {
    match CONFIG.common.meta_store.as_str() {
        "sqlite" => sqlite::create_table_index().await,
        "postgres" | "postgresql" => postgres::create_table_index().await,
        "dynamo" | "dynamodb" => dynamo::create_table_index().await,
        _ => sqlite::create_table_index().await,
    }
}

#[inline]
pub async fn add(file: &str, meta: &FileMeta) -> Result<()> {
    CLIENT.add(file, meta).await
}

#[inline]
pub async fn remove(file: &str) -> Result<()> {
    CLIENT.remove(file).await
}

#[inline]
pub async fn batch_add(files: &[FileKey]) -> Result<()> {
    CLIENT.batch_add(files).await
}

#[inline]
pub async fn batch_remove(files: &[String]) -> Result<()> {
    CLIENT.batch_remove(files).await
}

#[inline]
pub async fn get(file: &str) -> Result<FileMeta> {
    CLIENT.get(file).await
}

#[inline]
pub async fn contains(file: &str) -> Result<bool> {
    CLIENT.contains(file).await
}

#[inline]
pub async fn list() -> Result<Vec<(String, FileMeta)>> {
    CLIENT.list().await
}

#[inline]
pub async fn query(
    org_id: &str,
    stream_type: StreamType,
    stream_name: &str,
    time_level: PartitionTimeLevel,
    time_range: (i64, i64),
) -> Result<Vec<(String, FileMeta)>> {
    CLIENT
        .query(org_id, stream_type, stream_name, time_level, time_range)
        .await
}

#[inline]
pub async fn get_max_pk_value() -> Result<i64> {
    CLIENT.get_max_pk_value().await
}

#[inline]
pub async fn stats(
    org_id: &str,
    stream_type: Option<StreamType>,
    stream_name: Option<&str>,
    pk_value: Option<(i64, i64)>,
) -> Result<Vec<(String, StreamStats)>> {
    CLIENT
        .stats(org_id, stream_type, stream_name, pk_value)
        .await
}

#[inline]
pub async fn get_stream_stats(
    org_id: &str,
    stream_type: Option<StreamType>,
    stream_name: Option<&str>,
) -> Result<Vec<(String, StreamStats)>> {
    CLIENT
        .get_stream_stats(org_id, stream_type, stream_name)
        .await
}

#[inline]
pub async fn set_stream_stats(org_id: &str, streams: &[(String, StreamStats)]) -> Result<()> {
    CLIENT.set_stream_stats(org_id, streams).await
}

#[inline]
pub async fn reset_stream_stats_min_ts(org_id: &str, stream: &str, min_ts: i64) -> Result<()> {
    CLIENT
        .reset_stream_stats_min_ts(org_id, stream, min_ts)
        .await
}

#[inline]
pub async fn len() -> usize {
    CLIENT.len().await
}

#[inline]
pub async fn is_empty() -> bool {
    CLIENT.is_empty().await
}

#[inline]
pub async fn clear() -> Result<()> {
    CLIENT.clear().await
}

/// parse file key to get stream_key, date_key, file_name
pub fn parse_file_key_columns(key: &str) -> Result<(String, String, String)> {
    // eg: files/default/logs/olympics/2022/10/03/10/6982652937134804993_1.parquet
    let columns = key.splitn(9, '/').collect::<Vec<&str>>();
    if columns.len() < 9 {
        return Err(Error::Message(format!(
            "[file_list] Invalid file path: {}",
            key
        )));
    }
    // let _ = columns[0].to_string(); // files/
    let stream_key = format!("{}/{}/{}", columns[1], columns[2], columns[3]);
    let date_key = format!(
        "{}/{}/{}/{}",
        columns[4], columns[5], columns[6], columns[7]
    );
    let file_name = columns[8].to_string();
    Ok((stream_key, date_key, file_name))
}

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct FileRecord {
    pub stream: String,
    pub date: String,
    pub file: String,
    pub deleted: bool,
    pub min_ts: i64,
    pub max_ts: i64,
    pub records: i64,
    pub original_size: i64,
    pub compressed_size: i64,
}

impl From<&FileRecord> for FileMeta {
    fn from(record: &FileRecord) -> Self {
        Self {
            min_ts: record.min_ts,
            max_ts: record.max_ts,
            records: record.records,
            original_size: record.original_size,
            compressed_size: record.compressed_size,
        }
    }
}

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct StatsRecord {
    pub stream: String,
    pub file_num: i64,
    pub min_ts: i64,
    pub max_ts: i64,
    pub records: i64,
    pub original_size: i64,
    pub compressed_size: i64,
}

impl From<&StatsRecord> for StreamStats {
    fn from(record: &StatsRecord) -> Self {
        Self {
            created_at: 0,
            doc_time_min: record.min_ts,
            doc_time_max: record.max_ts,
            doc_num: record.records,
            file_num: record.file_num,
            storage_size: record.original_size as f64,
            compressed_size: record.compressed_size as f64,
        }
    }
}

impl From<&HashMap<String, AttributeValue>> for StatsRecord {
    fn from(data: &HashMap<String, AttributeValue>) -> Self {
        StatsRecord {
            stream: data.get("stream").unwrap().as_s().unwrap().to_string(),
            file_num: data
                .get("file_num")
                .unwrap()
                .as_n()
                .unwrap()
                .parse::<i64>()
                .unwrap(),
            min_ts: data
                .get("min_ts")
                .unwrap()
                .as_n()
                .unwrap()
                .parse::<i64>()
                .unwrap(),
            max_ts: data
                .get("max_ts")
                .unwrap()
                .as_n()
                .unwrap()
                .parse::<i64>()
                .unwrap(),
            records: data
                .get("records")
                .unwrap()
                .as_n()
                .unwrap()
                .parse::<i64>()
                .unwrap(),
            original_size: data
                .get("original_size")
                .unwrap()
                .as_n()
                .unwrap()
                .parse::<i64>()
                .unwrap(),
            compressed_size: data
                .get("compressed_size")
                .unwrap()
                .as_n()
                .unwrap()
                .parse::<i64>()
                .unwrap(),
        }
    }
}
