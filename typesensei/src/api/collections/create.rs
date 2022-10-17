use std::iter::once;

use crate::{
    api::CollectionResponse,
    schema::{CollectionSchema, Field},
    Client, Error,
};
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct CreateCollection<'a> {
    client: &'a Client,
    schema: CollectionSchema<'a>,
}

impl<'a> CreateCollection<'a> {
    pub(crate) fn new(client: &'a Client, name: &'a str) -> Self {
        Self {
            client,
            schema: CollectionSchema::new(name),
        }
    }

    pub fn field(mut self, field: Field<'a>) -> Self {
        self.schema = self.schema.field(field);
        self
    }

    pub fn default_sorting_field(mut self, default_sorting_field: &'a str) -> Self {
        self.schema = self.schema.default_sorting_field(default_sorting_field);
        self
    }

    #[instrument]
    pub async fn create(&self) -> Result<CollectionResponse, Error> {
        let ret = self.client.post((once(super::PATH), &self.schema)).await?;

        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let client = Client::new("localhost:8282", "xyz");
        let create = client
            .collections()
            .schema("ex_collection")
            .field(Field::string("field0"))
            .field(Field::int32("field1").facet(true))
            .default_sorting_field("field1");
    }
}
