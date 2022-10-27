use crate::{Client, Error, __priv::TypesenseReq};
use std::{fmt, iter::once, marker::PhantomData};
use tracing::instrument;

use super::CollectionResponse;

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

    fn path(&self) -> impl Iterator<Item = &'a str> + fmt::Debug {
        [PATH, T::schema_name()].into_iter()
    }

    #[instrument]
    pub async fn retreive(&self) -> Result<CollectionResponse, Error> {
        self.client.get(self.path()).await
    }

    #[instrument]
    pub async fn create(&self) -> Result<CollectionResponse, Error> {
        self.client.post((&T::schema(), once(PATH))).await
    }
}
