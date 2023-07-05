use crate::{Client, Error};
use base64::Engine;
use hmac::{Hmac, Mac};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sha2::Sha256;
use std::{
    iter::once,
    time::{Duration, SystemTime},
};
use tracing::instrument;

const PATH: &'static str = "keys";

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub description: String,
    pub actions: Vec<String>,
    pub collections: Vec<String>,
    pub value: Option<String>,
    pub expires_at: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
pub struct Keys<'a> {
    client: &'a Client,
}

impl<'a> Keys<'a> {
    pub(crate) fn new(client: &'a Client) -> Keys<'a> {
        Self { client }
    }

    #[instrument]
    pub async fn create(&self, key: &ApiKey) -> Result<ApiKey, Error> {
        self.client.post((key, once(PATH))).await
    }

    #[instrument]
    pub async fn retreive(&self, id: &str) -> Result<ApiKeyResponse, Error> {
        self.client.get([PATH, id].into_iter()).await
    }

    #[instrument]
    pub async fn retreive_all(&self) -> Result<ApiKeyResponses, Error> {
        self.client.get(once(PATH)).await
    }

    #[instrument]
    pub async fn delete(&self, id: &str) -> Result<ApiKeyDeleteResponse, Error> {
        self.client.delete([PATH, id].into_iter()).await
    }
}

pub fn generate_scoped_search_key(
    key: impl AsRef<str>,
    expire_in: Duration,
) -> GenerateScopedSearchKeyBuilder {
    GenerateScopedSearchKeyBuilder {
        key: key.as_ref().to_owned(),
        ..Default::default()
    }
    .expire_in(expire_in)
}

#[derive(Debug, Default, Clone)]
pub struct GenerateScopedSearchKeyBuilder {
    key: String,
    filters: GenerateScopedSearchKey,
}

#[derive(Debug, Default, Clone, Serialize)]
#[skip_serializing_none]
pub struct GenerateScopedSearchKey {
    query_by: Option<String>,
    filter_by: Option<String>,
    exclude_fields: Option<String>,
    limit_hits: Option<usize>,
    expires_at: u64,
}

impl GenerateScopedSearchKeyBuilder {
    pub fn expire_in(mut self, expire_in: Duration) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let expire_at = now + expire_in;
        self.filters.expires_at = expire_at.as_secs();

        self
    }

    pub fn query_by<'a>(mut self, query_by: impl IntoIterator<Item = &'a str>) -> Self {
        self.filters
            .query_by
            .replace(query_by.into_iter().join(","));

        self
    }

    pub fn filter_by(mut self, filter_by: impl AsRef<str>) -> Self {
        self.filters
            .filter_by
            .replace(filter_by.as_ref().to_owned());

        self
    }

    pub fn limit_hits(mut self, limit_hits: usize) -> Self {
        self.filters.limit_hits.replace(limit_hits);

        self
    }

    pub fn exclude_fields<'a>(mut self, fields: impl IntoIterator<Item = &'a str>) -> Self {
        self.filters
            .exclude_fields
            .replace(fields.into_iter().join(","));

        self
    }

    pub fn build(self) -> Result<String, Error> {
        let params = serde_json::to_string(&self.filters).unwrap();

        let mut mac = Hmac::<Sha256>::new_from_slice(self.key.as_bytes()).unwrap();
        mac.update(params.as_bytes());
        let result = mac.finalize();

        let standard = base64::engine::general_purpose::STANDARD;
        let digest = standard.encode(result.into_bytes());

        let key_prefix = &self.key.as_str()[0..4];
        let raw_scoped_key = format!("{}{}{}", digest, key_prefix, params);

        Ok(standard.encode(raw_scoped_key.as_bytes()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyDeleteResponse {
    pub id: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyResponses {
    keys: Vec<ApiKeyResponse>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyResponse {
    pub id: usize,
    pub description: String,
    pub actions: Vec<String>,
    pub collections: Vec<String>,
    pub value_prefix: String,
}
