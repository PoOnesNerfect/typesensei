use reqwest::header::InvalidHeaderValue;
use schema::FieldType;
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use std::fmt;

pub use typesensei_derive::Typesense;

mod client;
mod field_state;
pub use client::*;
pub use field_state::*;

pub mod api;
pub mod schema;

pub(crate) mod __priv {
    use super::Typesense;
    use serde::{Deserialize, Serialize};
    use std::fmt;

    pub trait TypesenseReq
    where
        Self: Typesense + fmt::Debug + Serialize,
        for<'de> Self: Deserialize<'de>,
    {
    }

    impl<T> TypesenseReq for T
    where
        T: Typesense + fmt::Debug + Serialize,
        for<'de> T: Deserialize<'de>,
    {
    }
}

pub trait Typesense: Sized {
    type Model: TypesenseModel + From<Self>;
    type DocumentId: fmt::Display + fmt::Debug + TypesenseField;

    fn schema_name() -> &'static str;

    fn schema() -> schema::CollectionSchema<'static>;

    fn model() -> Self::Model {
        Default::default()
    }
}

impl Typesense for serde_json::Value {
    type Model = Self;
    type DocumentId = String;

    fn schema_name() -> &'static str {
        "json"
    }

    fn schema() -> schema::CollectionSchema<'static> {
        schema::CollectionSchema::new(Self::schema_name()).field(schema::Field::auto(".*"))
    }
}

pub trait TypesenseModel
where
    Self: fmt::Debug + Default + Serialize,
    for<'de> Self: Deserialize<'de>,
{
}

impl TypesenseModel for serde_json::Value {}

pub trait TypesenseField {
    fn field_type() -> FieldType;
}

impl<T: TypesenseField> TypesenseField for &T {
    fn field_type() -> FieldType {
        T::field_type()
    }
}

impl<T: TypesenseField> TypesenseField for Option<T> {
    fn field_type() -> FieldType {
        T::field_type()
    }
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
impl_field!(u32, u64, usize, i64, isize => FieldType::Int64, FieldType::Int64Array);
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
    #[snafu(display("`{name}` is an extension and should not be used as document on its own"))]
    ExtensionCheck { name: &'static str },
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
