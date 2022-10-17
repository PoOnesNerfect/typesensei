use reqwest::header::InvalidHeaderValue;
use schema::FieldType;
use serde::{de::DeserializeOwned, Serialize};
use snafu::Snafu;
use std::fmt;

pub use typesensei_derive::Typesense;

pub mod api;
mod client;
pub mod schema;

pub use client::*;

pub trait Typesense: fmt::Debug + Serialize + DeserializeOwned {
    type DocumentId: fmt::Display + fmt::Debug + TypesenseField;

    fn schema_name() -> &'static str;

    fn schema() -> schema::CollectionSchema<'static>;
}

impl Typesense for serde_json::Value {
    type DocumentId = i32;

    fn schema_name() -> &'static str {
        "json"
    }

    fn schema() -> schema::CollectionSchema<'static> {
        schema::CollectionSchema::new(Self::schema_name()).field(schema::Field::auto(".*"))
    }
}

pub trait TypesenseField {
    fn field_type() -> FieldType;
}

macro_rules! impl_field {
    ($($t:ty),* => $n:expr, $a:expr) => {
        $(
            impl TypesenseField for $t {
                fn field_type() -> FieldType {
                    $n
                }
            }

            impl TypesenseField for Vec<$t> {
                fn field_type() -> FieldType {
                    $a
                }
            }
        )*
    };
}
impl_field!(u8, u16, i8, i16, i32 => FieldType::Int32, FieldType::Int32Array);
impl_field!(u32, i64, isize => FieldType::Int64, FieldType::Int64Array);
impl_field!(f32, f64 => FieldType::Float, FieldType::FloatArray);
impl_field!(&str, String => FieldType::String, FieldType::StringArray);
impl_field!(bool => FieldType::Bool, FieldType::BoolArray);

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
#[snafu(module)]
pub enum Error {
    #[snafu(display("Failed GET request"))]
    GetFailed { source: reqwest::Error },
    #[snafu(display("Failed POST request"))]
    PostFailed { source: reqwest::Error },
    #[snafu(display("Failed to deserialize response body as json"))]
    DeserializeResponse { source: reqwest::Error },
    #[snafu(display("Failed to serialize document {document:?} to json"))]
    DocumentToJson {
        document: String,
        source: serde_json::Error,
    },
    #[snafu(display("API Key not found"))]
    ApiKeyNotFound,
    #[snafu(display("Hostname not found"))]
    HostnameNotFound,
    #[snafu(display("API Key ({api_key}) is invalid"))]
    InvalidApiKey {
        api_key: String,
        source: InvalidHeaderValue,
    },
    #[snafu(display("ReqwestBuilder failed to build"))]
    ReqwestBuilderFailed { source: reqwest::Error },
}
