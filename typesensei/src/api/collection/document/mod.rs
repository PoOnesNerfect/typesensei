use super::Collection;
use crate::{api::DocumentResponse, error::*, Client, Error, Typesense};
use bytes::{BufMut, BytesMut};
use snafu::ResultExt;
use std::{fmt, future::Future, iter::once};
use tracing::instrument;

type DocumentResult = Result<Vec<DocumentResponse>, Error>;
const PATH: &'static str = "documents";

mod action;
pub use action::*;
mod batch;
pub use batch::*;

#[derive(Debug, Clone, Copy)]
pub struct Documents<'a, T: Typesense> {
    collection: Collection<'a, T>,
}

impl<'a, T: Typesense> Documents<'a, T> {
    pub(crate) fn new(collection: Collection<'a, T>) -> Documents<'a, T> {
        Self { collection }
    }

    pub(crate) fn client(&self) -> &Client {
        &self.collection.client
    }

    pub fn path(&self) -> impl Iterator<Item = &'a str> + fmt::Debug {
        self.collection.path().into_iter().chain(once(PATH))
    }

    #[instrument]
    pub async fn retrieve(&self, id: &T::DocumentId) -> Result<T, Error> {
        let id = id.to_string();

        let path = self.path().chain(once(id.as_str()));

        let ret = self.client().get(path).await?;

        Ok(ret)
    }

    #[instrument]
    pub fn create(
        &'a self,
        document: &'a T,
    ) -> DocumentAction<'a, T, impl 'a + Future<Output = Result<(), Error>>> {
        DocumentAction::new(self, document, self.action(None, document))
    }

    #[instrument]
    pub fn upsert(
        &'a self,
        document: &'a T,
    ) -> DocumentAction<'a, T, impl 'a + Future<Output = Result<(), Error>>> {
        DocumentAction::new(
            self,
            document,
            self.action(Some(("action", "upsert")), document),
        )
    }

    #[instrument]
    pub fn update(
        &'a self,
        document: &'a T,
    ) -> DocumentAction<'a, T, impl 'a + Future<Output = Result<(), Error>>> {
        DocumentAction::new(
            self,
            document,
            self.action(Some(("action", "update")), document),
        )
    }

    #[instrument(skip(documents))]
    pub fn batch_create(
        &'a self,
        documents: &'a [&'a T],
    ) -> DocumentBatchAction<'a, T, impl 'a + Future<Output = DocumentResult>> {
        DocumentBatchAction::new(self, documents, self.batch_action(None, documents))
    }

    #[instrument(skip(documents))]
    pub fn batch_upsert(
        &'a self,
        documents: &'a [&'a T],
    ) -> DocumentBatchAction<'a, T, impl 'a + Future<Output = DocumentResult>> {
        DocumentBatchAction::new(
            self,
            documents,
            self.batch_action(Some(("action", "upsert")), documents),
        )
    }

    #[instrument(skip(documents))]
    pub fn batch_update(
        &'a self,
        documents: &'a [&'a T],
    ) -> DocumentBatchAction<'a, T, impl 'a + Future<Output = DocumentResult>> {
        DocumentBatchAction::new(
            self,
            documents,
            self.batch_action(Some(("action", "update")), documents),
        )
    }

    async fn action(
        &self,
        query: impl IntoIterator<Item = (&'a str, &'a str)> + fmt::Debug,
        document: &T,
    ) -> Result<(), Error> {
        self.client().post((self.path(), query, document)).await
    }

    async fn batch_action(
        &self,
        query: impl IntoIterator<Item = (&'a str, &'a str)> + fmt::Debug,
        documents: &[&T],
    ) -> DocumentResult {
        let path = self.path().into_iter().chain(once("import"));

        let mut writer = BytesMut::new().writer();

        for document in documents {
            serde_json::to_writer(&mut writer, document).with_context(|_| DocumentToJsonSnafu {
                document: format!("{document:?}"),
            })?;
        }

        let ret = self
            .client()
            .post_raw(path, query, writer.into_inner())
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
