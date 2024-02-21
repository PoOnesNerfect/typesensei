use serde::{Deserialize, Serialize};
use std::fmt;

mod client;

pub mod api;
pub mod partial;
pub mod schema;

pub use crate::field_trait::TypesenseField;
pub use api::keys::{generate_scoped_search_key, ApiKey};
pub use client::*;
pub use error::Error;
pub use partial::Partial;
pub use reqwest::{Client as Reqwest, ClientBuilder as ReqwestBuilder};
pub use typesensei_derive::{Partial, Typesense};

pub trait Typesense
where
    Self: fmt::Debug + Serialize + partial::Partial,
    for<'de> Self: Deserialize<'de>,
{
    fn schema<'a>(collection_name: &'a str) -> schema::CollectionSchema<'a>;

    fn partial() -> Self::Partial;
}

mod field_trait {
    use crate::schema::Field;

    pub trait TypesenseField {
        const TYPE: &'static str;
    }

    impl<T: TypesenseField> TypesenseField for &T {
        const TYPE: &'static str = T::TYPE;
    }

    impl<T: TypesenseField> TypesenseField for &mut T {
        const TYPE: &'static str = T::TYPE;
    }

    impl<T: TypesenseField> TypesenseField for Option<T> {
        const TYPE: &'static str = T::TYPE;
    }

    macro_rules! impl_field {
        ($($t:ty),* => $n:expr, $a:expr) => {
            $(
                impl TypesenseField for $t {
                    const TYPE: &'static str = $n;
                }

                impl TypesenseField for Vec<$t> {
                    const TYPE: &'static str = $a;
                }
            )*
        };
    }

    impl_field!(u8, u16, i8, i16, i32 => Field::INT32, Field::INT32_ARRAY);
    impl_field!(u32, u64, usize, i64, isize => Field::INT64, Field::INT64_ARRAY);
    impl_field!(f32, f64 => Field::FLOAT, Field::FLOAT_ARRAY);
    impl_field!(String => Field::STRING, Field::STRING_ARRAY);
    impl_field!(bool => Field::BOOL, Field::BOOL_ARRAY);
    impl_field!(serde_json::Value => Field::OBJECT, Field::OBJECT_ARRAY);

    impl<'a> TypesenseField for &'a str {
        const TYPE: &'static str = Field::STRING;
    }

    impl<'a> TypesenseField for Vec<&'a str> {
        const TYPE: &'static str = Field::STRING_ARRAY;
    }
}

mod error {
    use crate::api::ImportResponse;
    use reqwest::header::InvalidHeaderValue;
    use thiserror::Error;
    use tosserror::Toss;

    #[derive(Debug, Error, Toss)]
    #[visibility(pub(crate))]
    pub enum Error {
        #[error("Request failed to Typesense")]
        ActionFailed(#[source] reqwest::Error),
        #[error("Failed to deserialize response body as json")]
        DeserializeBody(#[source] reqwest::Error),
        #[error("Failed to deserialize text {text} as json")]
        DeserializeText {
            text: String,
            source: serde_json::Error,
        },
        #[error("Failed to parse response as either `message` or `{0}`")]
        ParseFailed(&'static str),
        #[error("Failed to serialize document {document:?} to json")]
        DocumentToJson {
            document: String,
            source: serde_json::Error,
        },
        #[error("Failed to {action} multiple documents: {errors:#?}")]
        BatchActionFailed {
            action: String,
            errors: Vec<(usize, ImportResponse)>,
        },
        #[error("{0}")]
        TypesenseError(String),
        #[error("API Key not found")]
        ApiKeyNotFound,
        #[error("Hostname not found")]
        HostnameNotFound,
        #[error("API Key ({api_key}) is invalid")]
        InvalidApiKey {
            api_key: String,
            source: InvalidHeaderValue,
        },
        #[error("ReqwestBuilder failed to build")]
        ReqwestBuilderFailed(#[source] reqwest::Error),
    }
}
