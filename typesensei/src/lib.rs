use reqwest::header::InvalidHeaderValue;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use snafu::Snafu;

mod client;
pub use client::*;

pub mod api;
pub mod schema;
pub mod state;

pub mod traits {
    pub use crate::{
        field_trait::TypesenseField, model_trait::TypesenseModel, query_trait::TypesenseQuery,
    };
}

pub use typesensei_derive::Typesense;

pub trait Typesense: Sized {
    type Model: traits::TypesenseModel + From<Self>;
    type Query: traits::TypesenseQuery;

    fn schema_name() -> &'static str;

    fn schema() -> schema::CollectionSchema<'static>;

    fn model() -> Self::Model {
        Default::default()
    }

    fn query() -> Self::Query {
        Default::default()
    }
}

impl Typesense for serde_json::Value {
    type Model = Self;
    type Query = state::QueryBuilder;

    fn schema_name() -> &'static str {
        "json"
    }

    fn schema() -> schema::CollectionSchema<'static> {
        schema::CollectionSchema::new(Self::schema_name()).field(schema::Field::auto(".*"))
    }
}

mod model_trait {
    use serde::{Deserialize, Serialize};
    use std::fmt;

    pub trait TypesenseModel
    where
        Self: fmt::Debug + Default + Serialize,
        for<'de> Self: Deserialize<'de>,
    {
    }

    impl TypesenseModel for serde_json::Value {}
}

macro_rules! impl_search_query {
    ($($f:ident : $p:expr),* $(,)?) => {
        #[skip_serializing_none]
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct SearchQuery {
            pub q: String,
            $(
                pub $f : Option<String>,
            )*
        }

        impl SearchQuery {
            pub fn query_pairs(&self) -> [(&'static str, Option<&str>); impl_search_query!(@n $($f),*)] {
                [
                    ("q", Some(&self.q)),
                    $(
                        (stringify!($f), self. $f .as_ref().map(|s| s.as_str())),
                    )*
                ]
            }
        }

        mod query_trait {
            use crate::{SearchQuery, state::OrderedState};
            use std::{fmt::{self, Write}, rc::Rc, cell::RefCell};
            use paste::paste;

            pub trait TypesenseQuery
            where
                Self: fmt::Debug + Default,
            {
                fn q(mut self, query: String) -> SearchQuery {
                    paste! {$(
                        let mut $f = Vec::with_capacity(Self:: [<$f _len>] (&self));
                        Self:: [<extend_ $f>] (&mut self, &mut $f);
                        $f.sort();

                        let $f = (!$f.is_empty()).then(|| {
                            let mut ret = $f[0].to_string();
                            ret.reserve($f.len() * 5);
                            for x in $f.into_iter().skip(1) {
                                write!(&mut ret, $p, x).unwrap();
                            }
                            ret
                        });
                    )*}

                    SearchQuery {
                        q: query,
                        $(
                            $f,
                        )*
                    }
                }

                fn with_counter(counter: Rc<RefCell<u16>>) -> Self;
                paste! {$(
                    fn [<extend_ $f>] (this: &mut Self, extend: &mut Vec<OrderedState>);
                    fn [<$f _len>](this: &Self) -> usize;
                )*}
            }

            impl<T: TypesenseQuery> TypesenseQuery for Option<T> {
                fn with_counter(counter: Rc<RefCell<u16>>) -> Self {
                    Some(T::with_counter(counter))
                }

                paste!{$(
                    fn [<extend_ $f>] (this: &mut Self, extend: &mut Vec<OrderedState>) {
                        this.as_mut()
                            .map(|t| T:: [<extend_ $f>] (t, extend))
                            .unwrap_or_default()
                    }

                    fn [<$f _len>] (this: &Self) -> usize {
                        this.as_ref()
                            .map(|t| T:: [<$f _len>](t))
                            .unwrap_or_default()
                    }
                )*}
            }
        }
    };
    (@n) => (1);
    (@n $f:ident $(,)? $($g:ident),*) => {
        1 + impl_search_query!(@n $($g),*)
    };
}

impl_search_query! {
    query_by: ",{}",
    sort_by: ",{}",
    filter_by: "&&{}",
}

mod field_trait {
    use crate::schema::FieldType;
    use std::fmt;

    pub trait TypesenseField {
        type Type: TypesenseField + fmt::Display + fmt::Debug;

        fn field_type() -> FieldType;
    }

    impl<T: TypesenseField> TypesenseField for &T {
        type Type = <T as TypesenseField>::Type;

        fn field_type() -> FieldType {
            T::field_type()
        }
    }

    impl<T: TypesenseField> TypesenseField for &mut T {
        type Type = <T as TypesenseField>::Type;

        fn field_type() -> FieldType {
            T::field_type()
        }
    }

    impl<T: TypesenseField> TypesenseField for Option<T> {
        type Type = <T as TypesenseField>::Type;

        fn field_type() -> FieldType {
            T::field_type()
        }
    }

    macro_rules! impl_field {
    ($($t:ty),* => $n:expr, $a:expr) => {
        $(
            impl TypesenseField for $t {
                type Type = $t;

                fn field_type() -> FieldType {
                    $n
                }
            }

            impl TypesenseField for Vec<$t> {
                type Type = $t;

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
    impl_field!(String => FieldType::String, FieldType::StringArray);
    impl_field!(bool => FieldType::Bool, FieldType::BoolArray);

    impl<'a> TypesenseField for &'a str {
        type Type = &'a str;

        fn field_type() -> FieldType {
            FieldType::String
        }
    }

    impl<'a> TypesenseField for Vec<&'a str> {
        type Type = &'a str;

        fn field_type() -> FieldType {
            FieldType::StringArray
        }
    }
}

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

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
#[snafu(module)]
pub enum Error {
    #[snafu(display("Request failed to Typesense"))]
    ActionFailed { source: reqwest::Error },
    #[snafu(display("Failed to deserialize response body as json"))]
    DeserializeFailed { source: reqwest::Error },
    #[snafu(display("Failed to parse response as either `message` or `{ret_type}`"))]
    ParseFailed { ret_type: &'static str },
    #[snafu(display("Failed to serialize document {document:?} to json"))]
    DocumentToJson {
        document: String,
        source: serde_json::Error,
    },
    #[snafu(display("{message}"))]
    TypesenseError { message: String },
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
