use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{collections::HashMap, fmt};

pub use api::keys::{generate_scoped_search_key, ApiKey};
pub use client::*;
pub use error::Error;
pub use reqwest::{Client as Reqwest, ClientBuilder as ReqwestBuilder};

pub mod api;
mod client;
pub mod schema;
mod state;
pub use state::*;

pub mod traits {
    pub use crate::{
        field_trait::TypesenseField, model_trait::TypesenseModel, query_trait::TypesenseQuery,
    };
}

pub use typesensei_derive::Typesense;
pub trait Typesense: Sized {
    type Model: traits::TypesenseModel + From<Self>;
    // type Query: traits::TypesenseQuery;

    fn schema_name() -> String;

    fn schema() -> schema::CollectionSchema<'static>;

    fn model() -> Self::Model {
        Default::default()
    }

    // fn query() -> Self::Query {
    //     Default::default()
    // }
}

impl Typesense for serde_json::Value {
    type Model = Self;
    // type Query = state::QueryBuilder;

    fn schema_name() -> String {
        "json".to_owned()
    }

    fn schema() -> schema::CollectionSchema<'static> {
        schema::CollectionSchema::new(Self::schema_name()).field(schema::Field::auto(".*"))
    }
}

impl<K, V> Typesense for HashMap<K, V>
where
    K: AsRef<str>,
    Self: fmt::Debug + Default + Serialize,
    for<'de> Self: Deserialize<'de>,
{
    type Model = Self;
    // type Query = state::QueryBuilder;

    fn schema_name() -> String {
        "map".to_owned()
    }

    fn schema() -> schema::CollectionSchema<'static> {
        schema::CollectionSchema::new(Self::schema_name()).field(schema::Field::auto(".*"))
    }
}

mod model_trait {
    use serde::{Deserialize, Serialize};
    use std::{collections::HashMap, fmt};

    pub trait TypesenseModel
    where
        Self: fmt::Debug + Default + Serialize,
        for<'de> Self: Deserialize<'de>,
    {
    }

    impl TypesenseModel for serde_json::Value {}

    impl<K, V> TypesenseModel for HashMap<K, V>
    where
        K: AsRef<str>,
        Self: fmt::Debug + Serialize,
        for<'de> Self: Deserialize<'de>,
    {
    }
}

mod field_trait {
    use crate::schema::Field;
    use std::fmt;

    pub trait TypesenseField {
        type Type: TypesenseField + fmt::Display + fmt::Debug;

        fn field_type() -> &'static str;
    }

    impl<T: TypesenseField> TypesenseField for &T {
        type Type = <T as TypesenseField>::Type;

        fn field_type() -> &'static str {
            T::field_type()
        }
    }

    impl<T: TypesenseField> TypesenseField for &mut T {
        type Type = <T as TypesenseField>::Type;

        fn field_type() -> &'static str {
            T::field_type()
        }
    }

    impl<T: TypesenseField> TypesenseField for Option<T> {
        type Type = <T as TypesenseField>::Type;

        fn field_type() -> &'static str {
            T::field_type()
        }
    }

    macro_rules! impl_field {
        ($($t:ty),* => $n:expr, $a:expr) => {
            $(
                impl TypesenseField for $t {
                    type Type = $t;

                    fn field_type() -> &'static str {
                        $n
                    }
                }

                impl TypesenseField for Vec<$t> {
                    type Type = $t;

                    fn field_type() -> &'static str {
                        $a
                    }
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
        type Type = &'a str;

        fn field_type() -> &'static str {
            Field::STRING
        }
    }

    impl<'a> TypesenseField for Vec<&'a str> {
        type Type = &'a str;

        fn field_type() -> &'static str {
            Field::STRING_ARRAY
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSearchQuery {
    searches: Vec<SearchQuery>,
}

macro_rules! impl_search_query {
    ($($f:ident : $p:expr),* $(,)?) => {
        #[skip_serializing_none]
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct SearchQuery {
            pub collection: Option<String>,
            pub q: String,
            pub page: Option<String>,
            pub per_page: Option<String>,
            $(
                pub $f : Option<String>,
            )*
        }

        impl SearchQuery {
            pub fn page(mut self, page: usize) -> Self {
                self.page.replace(page.to_string());
                self
            }

            pub fn per_page(mut self, per_page: usize) -> Self {
                self.per_page.replace(per_page.to_string());
                self
            }
        }

        impl SearchQuery {
            pub fn empty_query_pairs() -> [(&'static str, Option<&'static str>); impl_search_query!(@n $($f),*) + 3] {
                [
                    ("q", None),
                    ("page", None),
                    ("per_page", None),
                    $(
                        (stringify!($f), None),
                    )*
                ]
            }

            pub fn query_pairs(&self) -> [(&'static str, Option<&str>); impl_search_query!(@n $($f),*) + 3] {
                [
                    ("q", Some(&self.q)),
                    ("page", self.page.as_ref().map(|p| p.as_str())),
                    ("per_page", self.per_page.as_ref().map(|p| p.as_str())),
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
                        collection: None,
                        page: None,
                        per_page: None,
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
    (@n) => (0);
    (@n $f:ident $(,)? $($g:ident),*) => {
        1 + impl_search_query!(@n $($g),*)
    };
}

impl_search_query! {
    query_by: ",{}",
    sort_by: ",{}",
    filter_by: "&&{}",
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

mod error {
    use crate::api::ImportResponse;
    use reqwest::header::InvalidHeaderValue;
    use tosserror::{Error, Toss};

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
