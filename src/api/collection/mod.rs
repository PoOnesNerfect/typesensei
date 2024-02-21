use crate::{schema::OwnedField, Client, Error, Typesense};
use std::{iter::once, marker::PhantomData};
use tracing::instrument;

use super::{documents::Documents, CollectionResponse, CollectionUpdate};

const PATH: &'static str = "collections";

#[derive(Debug, Clone)]
pub struct Collection<'a, T: Typesense> {
    client: &'a Client,
    collection_name: &'a str,
    _phantom: PhantomData<T>,
}

impl<'a, T: Typesense> Collection<'a, T> {
    pub(crate) fn new(client: &'a Client, collection_name: &'a str) -> Collection<'a, T> {
        Self {
            client,
            collection_name,
            _phantom: PhantomData,
        }
    }

    #[instrument(skip(self))]
    pub async fn documents(&self) -> Documents<'_, T> {
        Documents::new(self.client, self.collection_name.as_ref())
    }

    #[instrument(skip(self))]
    pub async fn retreive(&self) -> Result<CollectionResponse, Error> {
        self.client.get([PATH, self.collection_name.as_ref()]).await
    }

    #[instrument(skip(self))]
    pub async fn create(&self) -> Result<CollectionResponse, Error> {
        self.client
            .post((&T::schema(self.collection_name.as_ref()), once(PATH)))
            .await
    }

    #[instrument(skip(self))]
    pub async fn update(&self, fields: Vec<OwnedField>) -> Result<CollectionUpdate, Error> {
        self.client
            .patch((
                CollectionUpdate { fields },
                [PATH, self.collection_name.as_ref()],
            ))
            .await
    }

    #[instrument(skip(self))]
    pub async fn delete(&self) -> Result<CollectionResponse, Error> {
        self.client
            .delete([PATH, self.collection_name.as_ref()])
            .await
    }
}
