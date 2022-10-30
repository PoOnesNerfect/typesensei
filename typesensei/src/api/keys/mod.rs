use crate::{Client, Error};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::iter::once;
use tracing::instrument;

const PATH: &'static str = "keys";

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
    pub async fn retreive_all(&self) -> Result<ApiKeyResponse, Error> {
        self.client.get(once(PATH)).await
    }

    #[instrument]
    pub async fn delete(&self, id: &str) -> Result<ApiKeyDeleteResponse, Error> {
        self.client.delete([PATH, id].into_iter()).await
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub description: String,
    pub actions: Vec<ApiAction>,
    pub collections: Vec<String>,
    pub value: Option<String>,
    pub expires_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiAction {
    All,
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
    pub actions: Vec<ApiAction>,
    pub collections: Vec<String>,
    pub value_prefix: String,
}
