use super::{Client, NodeConfig, CONTENT_TYPE};
use crate::Error;
use reqwest::header::{HeaderMap, HeaderValue};
use std::env;
use tracing::instrument;

pub const TYPESENSE_API_KEY_HEADER_NAME: &str = "X-TYPESENSE-API-KEY";
pub const JSON_CONTENT_TYPE: HeaderValue = HeaderValue::from_static("application/json");

#[derive(Debug)]
pub struct ClientBuilder {
    hostname: Option<String>,
    reqwest_builder: Option<reqwest::ClientBuilder>,
    api_key: Option<String>,
    nodes: Vec<NodeConfig>,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            reqwest_builder: None,
            hostname: env::var("TYPESENSE_HOSTNAME").ok(),
            api_key: env::var("TYPESENSE_API_KEY").ok(),
            nodes: Vec::new(),
        }
    }

    pub fn hostname(mut self, hostname: impl ToString) -> Self {
        self.hostname.replace(hostname.to_string());
        self
    }

    pub fn api_key(mut self, api_key: impl ToString) -> Self {
        self.api_key.replace(api_key.to_string());
        self
    }

    pub fn nodes(mut self, nodes: impl IntoIterator<Item = impl Into<NodeConfig>>) -> Self {
        self.nodes.extend(nodes.into_iter().map(|n| n.into()));
        self
    }

    pub fn reqwest_builder(mut self, builder: reqwest::ClientBuilder) -> Self {
        self.reqwest_builder.replace(builder);
        self
    }

    #[instrument]
    pub fn build(self) -> Result<Client, Error> {
        let api_key = self.api_key.ok_or(Error::ApiKeyNotFound)?;
        let hostname = self.hostname.ok_or(Error::HostnameNotFound)?;

        let mut builder = self.reqwest_builder.unwrap_or_default();

        let mut header_map = HeaderMap::new();
        header_map.insert(
            TYPESENSE_API_KEY_HEADER_NAME,
            HeaderValue::from_str(&api_key).map_err(|e| Error::InvalidApiKey {
                api_key: api_key.to_owned(),
                source: e,
            })?,
        );
        header_map.insert(CONTENT_TYPE, JSON_CONTENT_TYPE);
        builder = builder.default_headers(header_map);

        let reqwest = builder.build().map_err(Error::ReqwestBuilderFailed)?;

        Ok(Client {
            reqwest,
            api_key: api_key.into(),
            hostname: hostname.into(),
        })
    }
}
