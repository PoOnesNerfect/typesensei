use crate::{Client, Error, __priv::TypesenseReq, schema::OwnedField};
use std::{borrow::Cow, iter::once, marker::PhantomData};
use tracing::instrument;

use super::{CollectionResponse, CollectionUpdate};

const PATH: &'static str = "collections";

#[derive(Debug, Clone)]
pub struct Collection<'a, T: TypesenseReq> {
    client: &'a Client,
    collection_name: Cow<'a, str>,
    _phantom: PhantomData<T>,
}

impl<'a, T: TypesenseReq> Collection<'a, T> {
    pub(crate) fn new(client: &'a Client) -> Collection<'a, T> {
        Self {
            client,
            collection_name: Cow::Owned(T::schema_name()),
            _phantom: PhantomData,
        }
    }

    pub(crate) fn new_with_name(client: &'a Client, collection_name: &'a str) -> Collection<'a, T> {
        Self {
            client,
            collection_name: Cow::Borrowed(collection_name),
            _phantom: PhantomData,
        }
    }

    #[instrument]
    pub async fn retreive(&self) -> Result<CollectionResponse, Error> {
        self.client.get([PATH, self.collection_name.as_ref()]).await
    }

    #[instrument]
    pub async fn create(&self) -> Result<CollectionResponse, Error> {
        self.client
            .post((
                &T::schema().with_name(self.collection_name.clone().into_owned()),
                once(PATH),
            ))
            .await
    }

    #[instrument]
    pub async fn update(&self, fields: Vec<OwnedField>) -> Result<CollectionUpdate, Error> {
        self.client
            .patch((
                CollectionUpdate { fields },
                [PATH, self.collection_name.as_ref()],
            ))
            .await
    }

    #[instrument]
    pub async fn delete(&self) -> Result<CollectionResponse, Error> {
        self.client
            .delete([PATH, self.collection_name.as_ref()])
            .await
    }
}
