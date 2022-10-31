use crate::{
    api::{collection::Collection, documents::Documents, CollectionResponse, keys::Keys},
    error::*,
    Error,
    __priv::TypesenseReq,
};
use bytes::Bytes;
use reqwest::{header::CONTENT_TYPE, Client as Reqwest, RequestBuilder};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use snafu::ResultExt;
use std::{
    any::type_name,
    fmt::{self, Write},
};
use std::{iter::once, sync::Arc};
use tracing::instrument;

pub mod builder;
mod node_config;
use builder::*;
pub use node_config::*;

type QueryPair<Q, const N: usize> = [(&'static str, Q); N];

#[derive(Debug, Clone)]
pub struct Client {
    pub reqwest: Reqwest,
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

    pub fn keys<'a>(&'a self) -> Keys<'a> {
        Keys::new(self)
    }

    pub async fn retrieve_collections(&self) -> Result<Vec<CollectionResponse>, Error> {
        self.get(once("collections")).await
    }

    pub fn collection<'a, T: TypesenseReq>(&'a self) -> Collection<'a, T> {
        Collection::new(self)
    }

    pub fn documents<'a, T: TypesenseReq>(&'a self) -> Documents<'a, T> {
        Documents::new(self)
    }
}

impl Client {
    #[instrument]
    pub async fn get<'a, B, P, Q, const N: usize, R>(
        &self,
        path_query_body: impl Into<BodyPathQuery<'a, B, P, Q, N>> + fmt::Debug,
    ) -> Result<R, Error>
    where
        B: Serialize + fmt::Debug,
        P: IntoIterator<Item = &'a str>,
        Q: Serialize + fmt::Debug,
        R: DeserializeOwned,
    {
        self.action(path_query_body, |url| self.reqwest.get(url))
            .await
    }

    #[instrument]
    pub async fn post<'a, B, P, Q, const N: usize, R>(
        &self,
        path_query_body: impl Into<BodyPathQuery<'a, B, P, Q, N>> + fmt::Debug,
    ) -> Result<R, Error>
    where
        B: Serialize + fmt::Debug,
        P: IntoIterator<Item = &'a str>,
        Q: Serialize + fmt::Debug,
        R: DeserializeOwned,
    {
        self.action(path_query_body, |url| self.reqwest.post(url))
            .await
    }

    #[instrument(skip(body))]
    pub(crate) async fn post_raw<'a, P, Q, const N: usize>(
        &'a self,
        path: P,
        body: impl Into<Bytes>,
        query: QueryPair<Q, N>,
    ) -> Result<String, Error>
    where
        P: IntoIterator<Item = &'a str> + fmt::Debug,
        Q: Serialize + fmt::Debug,
    {
        self.action_raw(path, body, query, |url| self.reqwest.post(url))
            .await
    }

    #[instrument]
    pub(crate) async fn patch<'a, B, P, Q, const N: usize, R>(
        &self,
        path_query_body: impl Into<BodyPathQuery<'a, B, P, Q, N>> + fmt::Debug,
    ) -> Result<R, Error>
    where
        B: Serialize + fmt::Debug,
        P: IntoIterator<Item = &'a str>,
        Q: Serialize + fmt::Debug,
        R: DeserializeOwned,
    {
        self.action(path_query_body, |url| self.reqwest.patch(url))
            .await
    }

    #[instrument(skip(body))]
    pub(crate) async fn patch_raw<'a, P, Q, const N: usize>(
        &'a self,
        path: P,
        body: impl Into<Bytes>,
        query: QueryPair<Q, N>,
    ) -> Result<String, Error>
    where
        P: IntoIterator<Item = &'a str> + fmt::Debug,
        Q: Serialize + fmt::Debug,
    {
        self.action_raw(path, body, query, |url| self.reqwest.patch(url))
            .await
    }

    #[instrument]
    pub(crate) async fn delete<'a, P, B, Q, const N: usize, R>(
        &self,
        path_query_body: impl Into<BodyPathQuery<'a, B, P, Q, N>> + fmt::Debug,
    ) -> Result<R, Error>
    where
        B: Serialize + fmt::Debug,
        P: IntoIterator<Item = &'a str>,
        Q: Serialize + fmt::Debug,
        R: DeserializeOwned,
    {
        self.action(path_query_body, |url| self.reqwest.delete(url))
            .await
    }

    async fn action<'a, B, P, Q, const N: usize, R, F>(
        &self,
        path_query_body: impl Into<BodyPathQuery<'a, B, P, Q, N>> + fmt::Debug,
        f: F,
    ) -> Result<R, Error>
    where
        B: Serialize + fmt::Debug,
        P: IntoIterator<Item = &'a str>,
        Q: Serialize + fmt::Debug,
        R: DeserializeOwned,
        F: FnOnce(&str) -> RequestBuilder,
    {
        let res: R = path_query_body
            .into()
            .build(self.hostname.as_str(), |url| f(url))
            .send()
            .await
            .context(ActionFailedSnafu)?
            .json()
            .await
            .context(DeserializeBodySnafu)?;

        // res.into_res()
        Ok(res)
    }

    async fn action_raw<'a, P, Q, const N: usize, F>(
        &'a self,
        path: P,
        body: impl Into<Bytes>,
        query: QueryPair<Q, N>,
        f: F,
    ) -> Result<String, Error>
    where
        P: IntoIterator<Item = &'a str> + fmt::Debug,
        Q: Serialize + fmt::Debug,
        F: FnOnce(&str) -> RequestBuilder,
    {
        let hostname = self.hostname.as_str();
        let path = path.into_iter();

        let mut url = String::with_capacity(hostname.len() + 1 + path.size_hint().0);
        write!(&mut url, "{}", hostname).unwrap();
        path.for_each(|p| {
            write!(&mut url, "/{}", p).unwrap();
        });

        let req = f(&url).body(body.into()).header(CONTENT_TYPE, "text/plain");

        req.query(query.as_ref())
            .send()
            .await
            .context(ActionFailedSnafu)?
            .text()
            .await
            .context(DeserializeBodySnafu)
    }
}

#[derive(Debug)]
pub struct BodyPathQuery<'a, B = (), P = Option<&'a str>, Q = &'a str, const N: usize = 0>
where
    B: Serialize + fmt::Debug,
    P: IntoIterator<Item = &'a str>,
    Q: Serialize + fmt::Debug,
{
    body: Option<B>,
    path: P,
    query: QueryPair<Q, N>,
}

impl<'a, B, P, Q, const N: usize> BodyPathQuery<'a, B, P, Q, N>
where
    B: Serialize + fmt::Debug,
    P: IntoIterator<Item = &'a str>,
    Q: Serialize + fmt::Debug,
{
    pub fn build<F>(self, hostname: &str, f: F) -> RequestBuilder
    where
        F: FnOnce(&str) -> RequestBuilder,
    {
        let Self { path, query, body } = self;
        let path = path.into_iter();

        let mut url = String::with_capacity(hostname.len() + 1 + path.size_hint().0);
        write!(&mut url, "{}", hostname).unwrap();
        path.for_each(|p| {
            write!(&mut url, "/{}", p).unwrap();
        });

        let req = f(&url);

        let req = if let Some(body) = body {
            req.json(&body)
        } else {
            req
        };

        req.query(query.as_ref())
    }
}

impl<'a, P> From<P> for BodyPathQuery<'a, (), P>
where
    P: IntoIterator<Item = &'a str>,
{
    fn from(path: P) -> Self {
        Self {
            path,
            query: [],
            body: None,
        }
    }
}

impl<'a, P, Q, const N: usize> From<(P, QueryPair<Q, N>)> for BodyPathQuery<'a, (), P, Q, N>
where
    P: IntoIterator<Item = &'a str>,
    Q: Serialize + fmt::Debug,
{
    fn from((path, query): (P, QueryPair<Q, N>)) -> Self {
        Self {
            path,
            query,
            body: None,
        }
    }
}

impl<'a, B, P> From<(B, P)> for BodyPathQuery<'a, B, P>
where
    B: Serialize + fmt::Debug,
    P: IntoIterator<Item = &'a str>,
{
    fn from((body, path): (B, P)) -> Self {
        Self {
            body: Some(body),
            path,
            query: [],
        }
    }
}

impl<'a, B, P, Q, const N: usize> From<(B, P, QueryPair<Q, N>)> for BodyPathQuery<'a, B, P, Q, N>
where
    B: Serialize + fmt::Debug,
    P: IntoIterator<Item = &'a str>,
    Q: Serialize + fmt::Debug,
{
    fn from((body, path, query): (B, P, QueryPair<Q, N>)) -> Self {
        Self {
            body: Some(body),
            path,
            query,
        }
    }
}

#[derive(Deserialize)]
struct TypesenseResult<T> {
    #[serde(flatten)]
    ret: Option<T>,
    message: Option<String>,
}

impl<T> TypesenseResult<T> {
    pub fn into_res(self) -> Result<T, Error> {
        if let Some(message) = self.message {
            Err(Error::TypesenseError { message })
        } else if let Some(ret) = self.ret {
            Ok(ret)
        } else {
            Err(Error::ParseFailed {
                ret_type: type_name::<T>(),
            })
        }
    }
}
