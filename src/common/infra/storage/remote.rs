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
use bytes::Bytes;
use futures::{stream::BoxStream, StreamExt};
use object_store::{
    limit::LimitStore, path::Path, Error, GetOptions, GetResult, ListResult, MultipartId,
    ObjectMeta, ObjectStore, Result,
};
use std::ops::Range;
use tokio::io::AsyncWrite;

use super::{format_key, CONCURRENT_REQUESTS};
use crate::common::infra::{config::CONFIG, metrics};

pub struct Remote {
    client: LimitStore<Box<dyn object_store::ObjectStore>>,
}

impl Default for Remote {
    fn default() -> Self {
        Self {
            client: LimitStore::new(init_client(), CONCURRENT_REQUESTS),
        }
    }
}

impl std::fmt::Debug for Remote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("storage for remote")
    }
}

impl std::fmt::Display for Remote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("storage for remote")
    }
}

#[async_trait]
impl ObjectStore for Remote {
    async fn put(&self, location: &Path, bytes: Bytes) -> Result<()> {
        let start = std::time::Instant::now();
        let file = location.to_string();
        let data_size = bytes.len();
        match self.client.put(&(format_key(&file).into()), bytes).await {
            Ok(_) => {
                // metrics
                let columns = file.split('/').collect::<Vec<&str>>();
                if columns[0] == "files" {
                    metrics::STORAGE_WRITE_BYTES
                        .with_label_values(&[columns[1], columns[3], columns[2]])
                        .inc_by(data_size as u64);

                    let time = start.elapsed().as_secs_f64();
                    metrics::STORAGE_TIME
                        .with_label_values(&[columns[1], columns[3], columns[2], "put"])
                        .inc_by(time);
                }
                Ok(())
            }
            Err(err) => {
                log::error!("s3 File upload error: {:?}", err);
                Err(err)
            }
        }
    }

    async fn put_multipart(
        &self,
        _location: &Path,
    ) -> Result<(MultipartId, Box<dyn AsyncWrite + Unpin + Send>)> {
        Err(Error::NotImplemented)
    }

    async fn abort_multipart(&self, _location: &Path, _multipart_id: &MultipartId) -> Result<()> {
        Err(Error::NotImplemented)
    }

    async fn get(&self, location: &Path) -> Result<GetResult> {
        let start = std::time::Instant::now();
        let file = location.to_string();
        let result = self.client.get(&(format_key(&file).into())).await?;

        // metrics
        let data = result.bytes().await?;
        let data_len = data.len();
        let columns = file.split('/').collect::<Vec<&str>>();
        if columns[0] == "files" {
            metrics::STORAGE_READ_BYTES
                .with_label_values(&[columns[1], columns[3], columns[2]])
                .inc_by(data_len as u64);

            let time = start.elapsed().as_secs_f64();
            metrics::STORAGE_TIME
                .with_label_values(&[columns[1], columns[3], columns[2], "get"])
                .inc_by(time);
        }

        Ok(GetResult::Stream(
            futures::stream::once(async move { Ok(data) }).boxed(),
        ))
    }

    async fn get_opts(&self, location: &Path, options: GetOptions) -> Result<GetResult> {
        let start = std::time::Instant::now();
        let file = location.to_string();
        let result = self
            .client
            .get_opts(&(format_key(&file).into()), options)
            .await?;

        // metrics
        let data = result.bytes().await?;
        let data_len = data.len();
        let columns = file.split('/').collect::<Vec<&str>>();
        if columns[0] == "files" {
            metrics::STORAGE_READ_BYTES
                .with_label_values(&[columns[1], columns[3], columns[2]])
                .inc_by(data_len as u64);

            let time = start.elapsed().as_secs_f64();
            metrics::STORAGE_TIME
                .with_label_values(&[columns[1], columns[3], columns[2], "get"])
                .inc_by(time);
        }

        Ok(GetResult::Stream(
            futures::stream::once(async move { Ok(data) }).boxed(),
        ))
    }

    async fn get_range(&self, location: &Path, range: Range<usize>) -> Result<Bytes> {
        let start = std::time::Instant::now();
        let file = location.to_string();
        let data = self
            .client
            .get_range(&(format_key(&file).into()), range)
            .await?;

        // metrics
        let data_len = data.len();
        let columns = file.split('/').collect::<Vec<&str>>();
        if columns[0] == "files" {
            metrics::STORAGE_READ_BYTES
                .with_label_values(&[columns[1], columns[3], columns[2]])
                .inc_by(data_len as u64);

            let time = start.elapsed().as_secs_f64();
            metrics::STORAGE_TIME
                .with_label_values(&[columns[1], columns[3], columns[2], "get"])
                .inc_by(time);
        }

        Ok(data)
    }

    async fn head(&self, _location: &Path) -> Result<ObjectMeta> {
        Err(Error::NotImplemented)
    }

    async fn delete(&self, location: &Path) -> Result<()> {
        let mut result: Result<()> = Ok(());
        for _ in 0..3 {
            result = self
                .client
                .delete(&(format_key(location.as_ref()).into()))
                .await;
            if result.is_ok() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        result
    }

    async fn list(&self, prefix: Option<&Path>) -> Result<BoxStream<'_, Result<ObjectMeta>>> {
        self.client
            .list(Some(&format_key(prefix.unwrap().as_ref()).into()))
            .await
    }

    async fn list_with_delimiter(&self, _prefix: Option<&Path>) -> Result<ListResult> {
        Err(Error::NotImplemented)
    }

    async fn copy(&self, _from: &Path, _to: &Path) -> Result<()> {
        Err(Error::NotImplemented)
    }

    async fn copy_if_not_exists(&self, _from: &Path, _to: &Path) -> Result<()> {
        Err(Error::NotImplemented)
    }
}

fn init_aws_config() -> object_store::Result<object_store::aws::AmazonS3> {
    let mut opts = object_store::ClientOptions::default()
        .with_connect_timeout(std::time::Duration::from_secs(CONFIG.s3.connect_timeout))
        .with_timeout(std::time::Duration::from_secs(CONFIG.s3.request_timeout))
        .with_allow_invalid_certificates(CONFIG.s3.allow_invalid_certificates)
        .with_allow_http(true);
    if CONFIG.s3.feature_http1_only {
        opts = opts.with_http1_only();
    }
    if CONFIG.s3.feature_http2_only {
        opts = opts.with_http2_only();
    }
    let mut builder = object_store::aws::AmazonS3Builder::from_env()
        .with_client_options(opts)
        .with_bucket_name(&CONFIG.s3.bucket_name)
        .with_virtual_hosted_style_request(CONFIG.s3.feature_force_path_style);
    if !CONFIG.s3.server_url.is_empty() {
        builder = builder.with_endpoint(&CONFIG.s3.server_url);
    }
    if !CONFIG.s3.region_name.is_empty() {
        builder = builder.with_region(&CONFIG.s3.region_name);
    }
    if !CONFIG.s3.access_key.is_empty() {
        builder = builder.with_access_key_id(&CONFIG.s3.access_key);
    }
    if !CONFIG.s3.secret_key.is_empty() {
        builder = builder.with_secret_access_key(&CONFIG.s3.secret_key);
    }
    builder.build()
}

fn init_azure_config() -> object_store::Result<object_store::azure::MicrosoftAzure> {
    let mut builder = object_store::azure::MicrosoftAzureBuilder::from_env()
        .with_client_options(
            object_store::ClientOptions::default()
                .with_connect_timeout(std::time::Duration::from_secs(CONFIG.s3.connect_timeout))
                .with_timeout(std::time::Duration::from_secs(CONFIG.s3.request_timeout))
                .with_allow_invalid_certificates(CONFIG.s3.allow_invalid_certificates),
        )
        .with_container_name(&CONFIG.s3.bucket_name);
    if !CONFIG.s3.access_key.is_empty() {
        builder = builder.with_account(&CONFIG.s3.access_key);
    }
    if !CONFIG.s3.secret_key.is_empty() {
        builder = builder.with_access_key(&CONFIG.s3.secret_key);
    }
    builder.build()
}

fn init_gcp_config() -> object_store::Result<object_store::gcp::GoogleCloudStorage> {
    let mut builder = object_store::gcp::GoogleCloudStorageBuilder::from_env()
        .with_client_options(
            object_store::ClientOptions::default()
                .with_connect_timeout(std::time::Duration::from_secs(CONFIG.s3.connect_timeout))
                .with_timeout(std::time::Duration::from_secs(CONFIG.s3.request_timeout))
                .with_allow_invalid_certificates(CONFIG.s3.allow_invalid_certificates),
        )
        .with_bucket_name(&CONFIG.s3.bucket_name);
    if !CONFIG.s3.access_key.is_empty() {
        builder = builder.with_service_account_path(&CONFIG.s3.access_key);
    }
    builder.build()
}

fn init_client() -> Box<dyn object_store::ObjectStore> {
    if CONFIG.common.print_key_config {
        log::info!("s3 init config: {:?}", CONFIG.s3);
    }

    match CONFIG.s3.provider.as_str() {
        "aws" | "s3" => match init_aws_config() {
            Ok(client) => Box::new(client),
            Err(e) => {
                panic!("s3 init config error: {:?}", e);
            }
        },
        "azure" => match init_azure_config() {
            Ok(client) => Box::new(client),
            Err(e) => {
                panic!("azure init config error: {:?}", e);
            }
        },
        "gcs" | "gcp" => match init_gcp_config() {
            Ok(client) => Box::new(client),
            Err(e) => {
                panic!("gcp init config error: {:?}", e);
            }
        },
        _ => match init_aws_config() {
            Ok(client) => Box::new(client),
            Err(e) => {
                panic!("{} init config error: {:?}", CONFIG.s3.provider, e);
            }
        },
    }
}
