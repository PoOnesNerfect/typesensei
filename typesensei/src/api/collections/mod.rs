use std::iter::once;

use crate::{schema::CollectionSchema, Client, Error};
use tracing::instrument;

pub mod create;
use create::*;

use super::CollectionResponse;

const PATH: &'static str = "collections";

#[derive(Debug, Clone, Copy)]
pub struct Collections<'a> {
    client: &'a Client,
}

impl<'a> Collections<'a> {
    pub(crate) fn new(client: &'a Client) -> Collections<'a> {
        Self { client }
    }

    pub fn schema(self, name: &'a str) -> CreateCollection<'a> {
        CreateCollection::new(self.client, name)
    }

    #[instrument]
    pub async fn create(&self, schema: &CollectionSchema<'a>) -> Result<CollectionResponse, Error> {
        self.client.post((once(PATH), &schema)).await
    }

    #[instrument]
    pub async fn retrieve(&self) -> Result<Vec<CollectionResponse>, Error> {
        self.client.get(once(PATH)).await
    }
}
