use super::Collection;
use crate::{api::DocumentResponse, error::*, Client, Error, __priv::TypesenseReq};
use bytes::{BufMut, BytesMut};
use snafu::ResultExt;
use std::{fmt, future::Future, iter::once};
use tracing::instrument;

type DocumentResult = Result<Vec<DocumentResponse>, Error>;
const PATH: &'static str = "documents";

mod batch;
pub use batch::*;

#[derive(Debug, Clone, Copy)]
pub struct Documents<'a, T: TypesenseReq> {
    collection: Collection<'a, T>,
}

impl<'a, T: TypesenseReq> Documents<'a, T> {
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
    pub async fn retrieve(&self, id: T::DocumentId) -> Result<T, Error> {
        let id = id.to_string();
        let path = self.path().chain(once(id.as_str()));

        let ret = self.client().get(path).await?;

        Ok(ret)
    }

    #[instrument]
    pub async fn create<M: fmt::Debug + Into<T::Model>>(&self, document: M) -> Result<T, Error> {
        self.client().post((self.path(), document.into())).await
    }

    #[instrument]
    pub async fn upsert<M: fmt::Debug + Into<T::Model>>(&self, document: M) -> Result<T, Error> {
        self.client()
            .post((self.path(), once(("action", "upsert")), document.into()))
            .await
    }

    #[instrument]
    pub async fn update<M: fmt::Debug + Into<T::Model>>(
        &self,
        id: T::DocumentId,
        document: M,
    ) -> Result<T, Error> {
        let document = document.into();
        let id = id.to_string();
        let path = self.path().chain(once(id.as_str()));

        self.client().patch((path, document)).await
    }

    #[instrument]
    pub async fn delete(&self, id: T::DocumentId) -> Result<(), Error> {
        let id = id.to_string();
        let path = self.path().chain(once(id.as_str()));

        self.client().delete(path).await
    }

    #[instrument(skip(documents))]
    pub fn batch_create(
        &'a self,
        documents: &'a [&'a T],
    ) -> DocumentBatchAction<'a, T, impl 'a + Future<Output = DocumentResult>> {
        DocumentBatchAction::new(self, None, documents, self.batch_action(None, documents))
    }

    #[instrument(skip(documents))]
    pub fn batch_upsert(
        &'a self,
        documents: &'a [&'a T],
    ) -> DocumentBatchAction<'a, T, impl 'a + Future<Output = DocumentResult>> {
        let action = "upsert";

        DocumentBatchAction::new(
            self,
            Some(action),
            documents,
            self.batch_action(Some(("action", action)), documents),
        )
    }

    #[instrument(skip(documents))]
    pub fn batch_update(
        &'a self,
        documents: &'a [&'a T],
    ) -> DocumentBatchAction<'a, T, impl 'a + Future<Output = DocumentResult>> {
        let action = "update";

        DocumentBatchAction::new(
            self,
            Some(action),
            documents,
            self.batch_action(Some(("action", action)), documents),
        )
    }

    #[instrument(skip(documents))]
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
