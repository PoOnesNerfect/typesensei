use super::ImportResponse;
use crate::{error::*, Client, Error, Typesense};
use bytes::{BufMut, BytesMut};
use std::{future::Future, io::Write, marker::PhantomData};
use tracing::instrument;

type BatchResult = Result<(), Error>;
type QueryPair<'a, const N: usize> = [(&'static str, Option<&'a str>); N];

mod batch;
pub use batch::*;

#[derive(Debug, Clone)]
pub struct Documents<'a, T: Typesense> {
    client: &'a Client,
    collection_name: &'a str,
    _phantom: PhantomData<T>,
}

impl<'a, T: Typesense> Documents<'a, T> {
    pub(crate) fn new(client: &'a Client, collection_name: &'a str) -> Documents<'a, T> {
        Self {
            client,
            collection_name,
            _phantom: PhantomData,
        }
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    #[instrument(skip(self))]
    pub async fn create(&self, document: &T) -> Result<T::Partial, Error> {
        self.client()
            .post((
                document,
                ["collections", self.collection_name.as_ref(), "documents"],
            ))
            .await
    }

    #[instrument(skip(self))]
    pub async fn retrieve(&self, id: &str) -> Result<T, Error> {
        let path = [
            "collections",
            self.collection_name.as_ref(),
            "documents",
            id,
        ];

        let ret = self.client().get(path).await?;

        Ok(ret)
    }

    #[instrument(skip(self))]
    pub async fn upsert(&self, document: &T) -> Result<T::Partial, Error> {
        let path = ["collections", self.collection_name.as_ref(), "documents"];

        self.client()
            .post((document, path, [("action", Some("upsert"))]))
            .await
    }

    #[instrument(skip(self))]
    pub async fn update(&self, id: &str, document: &T::Partial) -> Result<T::Partial, Error> {
        let path = [
            "collections",
            self.collection_name.as_ref(),
            "documents",
            id,
        ];

        self.client().patch((document, path)).await
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: &str) -> Result<T, Error> {
        let path = [
            "collections",
            self.collection_name.as_ref(),
            "documents",
            id,
        ];

        self.client().delete(path).await
    }

    #[instrument(skip(self, documents))]
    pub fn batch_create(
        &'a self,
        documents: &'a [T::Partial],
    ) -> DocumentBatchAction<'a, T, impl 'a + Future<Output = BatchResult>> {
        DocumentBatchAction::new(self, None, documents, self.batch_action([], documents))
    }

    #[instrument(skip(self, documents))]
    pub fn batch_upsert(
        &'a self,
        documents: &'a [T::Partial],
    ) -> DocumentBatchAction<'a, T, impl 'a + Future<Output = BatchResult>> {
        let action = Some("upsert");

        DocumentBatchAction::new(
            self,
            action,
            documents,
            self.batch_action([("action", action)], documents),
        )
    }

    #[instrument(skip(self, documents))]
    pub fn batch_update(
        &'a self,
        documents: &'a [T::Partial],
    ) -> DocumentBatchAction<'a, T, impl 'a + Future<Output = BatchResult>> {
        let action = Some("update");

        DocumentBatchAction::new(
            self,
            action,
            documents,
            self.batch_action([("action", action)], documents),
        )
    }

    #[instrument(skip(self, documents))]
    pub fn batch_emplace(
        &'a self,
        documents: &'a [T::Partial],
    ) -> DocumentBatchAction<'a, T, impl 'a + Future<Output = BatchResult>> {
        let action = Some("emplace");

        DocumentBatchAction::new(
            self,
            action,
            documents,
            self.batch_action([("action", action)], documents),
        )
    }

    #[instrument(skip(self, documents))]
    async fn batch_action<const N: usize>(
        &'a self,
        query: QueryPair<'a, N>,
        documents: &'a [T::Partial],
    ) -> BatchResult {
        let path = [
            "collections",
            self.collection_name.as_ref(),
            "documents",
            "import",
        ];

        let mut writer = BytesMut::new().writer();

        for document in documents {
            serde_json::to_writer(&mut writer, document)
                .toss_document_to_json_with(|| format!("{document:?}"))?;
            writer.write(&[b'\n']).expect("does not return Err ever");
        }

        let action = query
            .iter()
            .find(|q| q.0 == "action")
            .map(|q| q.1)
            .flatten()
            .unwrap_or("create");

        let body = self
            .client()
            .post_raw(path, writer.into_inner(), query)
            .await?;

        // info!("body: {body}");

        let mut res = Vec::with_capacity(documents.len());

        for line in body.lines() {
            res.push(serde_json::from_str(&line).toss_deserialize_text_with(|| line.to_owned())?);
        }

        import_into_res(action, res)
    }
}

fn import_into_res(action: &str, imports: Vec<ImportResponse>) -> Result<(), Error> {
    let mut errors = Vec::new();

    for (i, import) in imports.into_iter().enumerate() {
        if !import.success {
            errors.push((i, import));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(Error::BatchActionFailed {
            action: action.to_owned(),
            errors,
        })
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
