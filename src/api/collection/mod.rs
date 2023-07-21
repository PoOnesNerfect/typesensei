use crate::{Client, Error, __priv::TypesenseReq, schema::OwnedField};
use std::{iter::once, marker::PhantomData};
use tracing::instrument;

use super::{CollectionResponse, CollectionUpdate};

const PATH: &'static str = "collections";

#[derive(Debug, Clone, Copy)]
pub struct Collection<'a, T: TypesenseReq> {
    client: &'a Client,
    _phantom: PhantomData<T>,
}

impl<'a, T: TypesenseReq> Collection<'a, T> {
    pub(crate) fn new(client: &'a Client) -> Collection<'a, T> {
        Self {
            client,
            _phantom: PhantomData,
        }
    }

    #[instrument]
    pub async fn retreive(&self) -> Result<CollectionResponse, Error> {
        self.client.get([PATH, T::schema_name().as_str()]).await
    }

    #[instrument]
    pub async fn create(&self) -> Result<CollectionResponse, Error> {
        self.client.post((&T::schema(), once(PATH))).await
    }

    #[instrument]
    pub async fn update(&self, fields: Vec<OwnedField>) -> Result<CollectionUpdate, Error> {
        self.client
            .patch((
                CollectionUpdate { fields },
                [PATH, T::schema_name().as_str()],
            ))
            .await
    }

    #[instrument]
    pub async fn delete(&self) -> Result<CollectionResponse, Error> {
        self.client.delete([PATH, T::schema_name().as_str()]).await
    }
}
