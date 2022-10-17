use crate::{
    api::{collection::Collection, collections::Collections},
    error::*,
    Error, Typesense,
};
use bytes::Bytes;
use reqwest::{header::CONTENT_TYPE, Client as Reqwest, RequestBuilder};
use serde::{de::DeserializeOwned, Serialize};
use snafu::ResultExt;
use std::fmt;
use std::sync::Arc;
use tracing::instrument;

pub mod builder;
mod node_config;
use builder::*;
pub use node_config::*;

#[derive(Debug, Clone)]
pub struct Client {
    reqwest: Reqwest,
    hostname: Arc<String>,
    api_key: Arc<String>,
}

impl Client {
    pub fn new(hostname: &str, api_key: &str) -> Self {
        Self::builder()
            .api_key(api_key)
            .hostname(hostname)
            .build()
            .expect("Default Reqwest Client should build successfully")
    }

    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub fn hostname(&self) -> &str {
        &self.hostname
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn collection<'a, T: Typesense>(&'a self) -> Collection<'a, T> {
        Collection::new(self)
    }

    pub fn collections(&self) -> Collections {
        Collections::new(self)
    }
}

impl Client {
    #[instrument]
    pub(crate) async fn get<'a, P, Q, Q2, B, R>(
        &self,
        path_query_body: impl Into<PathQueryBody<'a, P, Q, Q2, B>> + fmt::Debug,
    ) -> Result<R, Error>
    where
        P: IntoIterator<Item = &'a str>,
        Q: IntoIterator<Item = Q2>,
        Q2: Serialize + fmt::Debug,
        B: Serialize + fmt::Debug,
        R: DeserializeOwned,
    {
        path_query_body
            .into()
            .build(self.hostname.as_ref().clone(), |url| self.reqwest.get(url))
            .send()
            .await
            .context(GetFailedSnafu)?
            .json()
            .await
            .context(DeserializeResponseSnafu)
    }

    #[instrument]
    pub(crate) async fn post<'a, P, Q, Q2, B, R>(
        &self,
        path_query_body: impl Into<PathQueryBody<'a, P, Q, Q2, B>> + fmt::Debug,
    ) -> Result<R, Error>
    where
        P: IntoIterator<Item = &'a str>,
        Q: IntoIterator<Item = Q2>,
        Q2: Serialize + fmt::Debug,
        B: Serialize + fmt::Debug,
        R: DeserializeOwned,
    {
        self.action(path_query_body, |url| self.reqwest.post(url))
            .await
    }

    #[instrument(skip(body))]
    pub(crate) async fn post_raw<'a, P, Q, Q2, R>(
        &self,
        path: P,
        query: Q,
        body: impl Into<Bytes>,
    ) -> Result<R, Error>
    where
        P: IntoIterator<Item = &'a str> + fmt::Debug,
        Q: IntoIterator<Item = Q2> + fmt::Debug,
        Q2: Serialize + fmt::Debug,
        R: DeserializeOwned,
    {
        self.action_raw(path, query, body, |url| self.reqwest.post(url))
            .await
    }

    #[instrument]
    pub(crate) async fn patch<'a, P, Q, Q2, B, R>(
        &self,
        path_query_body: impl Into<PathQueryBody<'a, P, Q, Q2, B>> + fmt::Debug,
    ) -> Result<R, Error>
    where
        P: IntoIterator<Item = &'a str>,
        Q: IntoIterator<Item = Q2>,
        Q2: Serialize + fmt::Debug,
        B: Serialize + fmt::Debug,
        R: DeserializeOwned,
    {
        self.action(path_query_body, |url| self.reqwest.patch(url))
            .await
    }

    #[instrument(skip(body))]
    pub(crate) async fn patch_raw<'a, P, Q, Q2, R>(
        &self,
        path: P,
        query: Q,
        body: impl Into<Bytes>,
    ) -> Result<R, Error>
    where
        P: IntoIterator<Item = &'a str> + fmt::Debug,
        Q: IntoIterator<Item = Q2> + fmt::Debug,
        Q2: Serialize + fmt::Debug,
        R: DeserializeOwned,
    {
        self.action_raw(path, query, body, |url| self.reqwest.patch(url))
            .await
    }

    #[instrument]
    pub(crate) async fn delete<'a, P, Q, Q2, B, R>(
        &self,
        path_query_body: impl Into<PathQueryBody<'a, P, Q, Q2, B>> + fmt::Debug,
    ) -> Result<R, Error>
    where
        P: IntoIterator<Item = &'a str>,
        Q: IntoIterator<Item = Q2>,
        Q2: Serialize + fmt::Debug,
        B: Serialize + fmt::Debug,
        R: DeserializeOwned,
    {
        self.action(path_query_body, |url| self.reqwest.delete(url))
            .await
    }

    #[instrument(skip(body))]
    pub(crate) async fn delete_raw<'a, P, Q, Q2, R>(
        &self,
        path: P,
        query: Q,
        body: impl Into<Bytes>,
    ) -> Result<R, Error>
    where
        P: IntoIterator<Item = &'a str> + fmt::Debug,
        Q: IntoIterator<Item = Q2> + fmt::Debug,
        Q2: Serialize + fmt::Debug,
        R: DeserializeOwned,
    {
        self.action_raw(path, query, body, |url| self.reqwest.delete(url))
            .await
    }

    async fn action<'a, P, Q, Q2, B, R, F>(
        &self,
        path_query_body: impl Into<PathQueryBody<'a, P, Q, Q2, B>> + fmt::Debug,
        f: F,
    ) -> Result<R, Error>
    where
        P: IntoIterator<Item = &'a str>,
        Q: IntoIterator<Item = Q2>,
        Q2: Serialize + fmt::Debug,
        B: Serialize + fmt::Debug,
        R: DeserializeOwned,
        F: FnOnce(&str) -> RequestBuilder,
    {
        path_query_body
            .into()
            .build(self.hostname.as_ref().clone(), |url| f(url))
            .send()
            .await
            .context(GetFailedSnafu)?
            .json()
            .await
            .context(DeserializeResponseSnafu)
    }

    async fn action_raw<'a, P, Q, Q2, R, F>(
        &self,
        path: P,
        query: Q,
        body: impl Into<Bytes>,
        f: F,
    ) -> Result<R, Error>
    where
        P: IntoIterator<Item = &'a str> + fmt::Debug,
        Q: IntoIterator<Item = Q2> + fmt::Debug,
        Q2: Serialize + fmt::Debug,
        R: DeserializeOwned,
        F: FnOnce(&str) -> RequestBuilder,
    {
        let mut url = self.hostname.as_ref().clone();

        for p in path {
            url.push('/');
            url += p;
        }

        let mut req = f(&url).body(body.into()).header(CONTENT_TYPE, "text/plain");

        for q in query {
            req = req.query(&q);
        }

        req.send()
            .await
            .context(GetFailedSnafu)?
            .json()
            .await
            .context(DeserializeResponseSnafu)
    }
}

#[derive(Debug)]
pub(crate) struct PathQueryBody<'a, P = Option<&'a str>, Q = Option<&'a str>, Q2 = &'a str, B = ()>
where
    P: IntoIterator<Item = &'a str>,
    Q: IntoIterator<Item = Q2>,
    Q2: Serialize + fmt::Debug,
    B: Serialize + fmt::Debug,
{
    path: P,
    query: Q,
    body: Option<B>,
}

impl<'a, P, Q, Q2, B> PathQueryBody<'a, P, Q, Q2, B>
where
    P: IntoIterator<Item = &'a str>,
    Q: IntoIterator<Item = Q2>,
    Q2: Serialize + fmt::Debug,
    B: Serialize + fmt::Debug,
{
    pub fn build<F>(self, hostname: String, f: F) -> RequestBuilder
    where
        F: FnOnce(&str) -> RequestBuilder,
    {
        let Self { path, query, body } = self;
        let mut url = hostname;

        for p in path {
            url.push('/');
            url += p;
        }

        let req = f(&url);

        let mut req = if let Some(body) = body {
            req.json(&body)
        } else {
            req
        };

        for q in query {
            req = req.query(&q);
        }

        req
    }
}

impl<'a, P> From<P> for PathQueryBody<'a, P>
where
    P: IntoIterator<Item = &'a str>,
{
    fn from(path: P) -> Self {
        Self {
            path,
            query: None,
            body: None,
        }
    }
}

impl<'a, P, Q, Q2> From<(P, Q)> for PathQueryBody<'a, P, Q, Q2>
where
    P: IntoIterator<Item = &'a str>,
    Q: IntoIterator<Item = Q2>,
    Q2: Serialize + fmt::Debug,
{
    fn from((path, query): (P, Q)) -> Self {
        Self {
            path,
            query,
            body: None,
        }
    }
}

impl<'a, P, B> From<(P, B)> for PathQueryBody<'a, P, Option<&'a str>, &'a str, B>
where
    P: IntoIterator<Item = &'a str>,
    B: Serialize + fmt::Debug,
{
    fn from((path, body): (P, B)) -> Self {
        Self {
            path,
            query: None,
            body: Some(body),
        }
    }
}

impl<'a, P, Q, Q2, B> From<(P, Q, B)> for PathQueryBody<'a, P, Q, Q2, B>
where
    P: IntoIterator<Item = &'a str>,
    Q: IntoIterator<Item = Q2>,
    Q2: Serialize + fmt::Debug,
    B: Serialize + fmt::Debug,
{
    fn from((path, query, body): (P, Q, B)) -> Self {
        Self {
            path,
            query,
            body: Some(body),
        }
    }
}
