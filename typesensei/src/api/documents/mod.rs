use crate::{error::*, Client, Error, SearchQuery, __priv::TypesenseReq};
use bytes::{BufMut, BytesMut};
use snafu::ResultExt;
use std::{fmt, future::Future, io::Write, iter::once, marker::PhantomData};
use tracing::instrument;

type DocumentResult = Result<Vec<ImportResponse>, Error>;
type QueryPair<'a, const N: usize> = [(&'static str, Option<&'a str>); N];

mod batch;
pub use batch::*;

use super::{ImportResponse, SearchResponse};

#[derive(Debug, Clone, Copy)]
pub struct Documents<'a, T: TypesenseReq> {
    client: &'a Client,
    _phantom: PhantomData<T>,
}

impl<'a, T: TypesenseReq> Documents<'a, T> {
    pub(crate) fn new(client: &'a Client) -> Documents<'a, T> {
        Self {
            client,
            _phantom: PhantomData,
        }
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    fn path(&self) -> impl Iterator<Item = &'a str> + fmt::Debug {
        ["collections", T::schema_name(), "documents"].into_iter()
    }

    #[instrument]
    pub async fn create(&self, document: &T::Model) -> Result<T::Model, Error> {
        self.client().post((document, self.path())).await
    }

    #[instrument]
    pub async fn retrieve(&self, id: &str) -> Result<T, Error> {
        let path = self.path().chain(once(id));

        let ret = self.client().get(path).await?;

        Ok(ret)
    }

    #[instrument]
    pub async fn search(&self, query: &SearchQuery) -> Result<SearchResponse<T>, Error> {
        let path = self.path().chain(once("search"));

        let ret = self.client().get((path, query.query_pairs())).await?;

        Ok(ret)
    }

    #[instrument]
    pub async fn upsert(&self, document: &T::Model) -> Result<T::Model, Error> {
        self.client()
            .post((document, self.path(), [("action", Some("upsert"))]))
            .await
    }

    #[instrument]
    pub async fn update(&self, id: &str, document: &T::Model) -> Result<T::Model, Error> {
        let path = self.path().chain(once(id));

        self.client().patch((document, path)).await
    }

    #[instrument]
    pub async fn delete(&self, id: &str) -> Result<T, Error> {
        let path = self.path().chain(once(id));

        self.client().delete(path).await
    }

    #[instrument(skip(documents))]
    pub fn batch_create(
        &'a self,
        documents: &'a [T::Model],
    ) -> DocumentBatchAction<'a, T, impl 'a + Future<Output = DocumentResult>> {
        DocumentBatchAction::new(self, None, documents, self.batch_action([], documents))
    }

    #[instrument(skip(documents))]
    pub fn batch_upsert(
        &'a self,
        documents: &'a [T::Model],
    ) -> DocumentBatchAction<'a, T, impl 'a + Future<Output = DocumentResult>> {
        let action = Some("upsert");

        DocumentBatchAction::new(
            self,
            action,
            documents,
            self.batch_action([("action", action)], documents),
        )
    }

    #[instrument(skip(documents))]
    pub fn batch_update(
        &'a self,
        documents: &'a [T::Model],
    ) -> DocumentBatchAction<'a, T, impl 'a + Future<Output = DocumentResult>> {
        let action = Some("update");

        DocumentBatchAction::new(
            self,
            action,
            documents,
            self.batch_action([("action", action)], documents),
        )
    }

    #[instrument(skip(documents))]
    pub fn batch_emplace(
        &'a self,
        documents: &'a [T::Model],
    ) -> DocumentBatchAction<'a, T, impl 'a + Future<Output = DocumentResult>> {
        let action = Some("emplace");

        DocumentBatchAction::new(
            self,
            action,
            documents,
            self.batch_action([("action", action)], documents),
        )
    }

    #[instrument(skip(documents))]
    async fn batch_action<const N: usize>(
        &'a self,
        query: QueryPair<'a, N>,
        documents: &'a [T::Model],
    ) -> DocumentResult {
        let path = self.path().into_iter().chain(once("import"));

        let mut writer = BytesMut::new().writer();

        for document in documents {
            serde_json::to_writer(&mut writer, document).with_context(|_| DocumentToJsonSnafu {
                document: format!("{document:?}"),
            })?;
            writer.write(&[b'\n']).expect("does not return Err ever");
        }

        let ret = self
            .client()
            .post_raw(path, writer.into_inner(), query)
            .await?;

        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actions() {
        let client = Client::new("hostname", "xyz");
        // client
        //     .collection()
        //     .documents()
        //     .create(&document)
        //     .dirty_values(dirty_values)
        //     .await;
    }
}
