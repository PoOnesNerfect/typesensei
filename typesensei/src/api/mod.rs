use crate::schema::FieldOwned;
use serde::{de::Visitor, Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub mod collection;
pub mod documents;
pub mod keys;

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionResponse {
    pub name: String,
    pub num_documents: usize,
    pub fields: Vec<FieldOwned>,
    pub default_sorting_field: String,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResponse {
    pub success: bool,
    pub error: Option<String>,
    pub document: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSearchResponse<T> {
    pub results: Vec<SearchResponse<T>>
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse<T> {
    pub facet_counts: Vec<usize>,
    pub found: usize,
    pub hits: Vec<SearchHit<T>>,
    pub out_of: usize,
    pub page: usize,
    pub request_params: SearchParams,
    pub search_cutoff: bool,
    pub search_time_ms: usize,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit<T> {
    pub document: T,
    pub highlights: Vec<SearchHighlight>,
    pub text_match: usize,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHighlight {
    pub field: String,
    pub indices: Option<Vec<usize>>,
    pub matched_tokens: Vec<MatchedToken>,
    pub snippet: Option<String>,
    pub snippets: Option<Vec<String>>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchParams {
    pub collection_name: String,
    pub per_page: usize,
    pub q: String,
}

#[derive(Debug, Clone)]
pub enum MatchedToken {
    Token(String),
    Tokens(Vec<String>),
}

impl MatchedToken {
    pub fn is_token(&self) -> bool {
        matches!(self, MatchedToken::Token(_))
    }

    pub fn is_tokens(&self) -> bool {
        matches!(self, MatchedToken::Tokens(_))
    }

    pub fn get_token(&self) -> Option<&String> {
        if let Self::Token(t) = self {
            Some(t)
        } else {
            None
        }
    }

    pub fn get_tokens(&self) -> Option<&Vec<String>> {
        if let Self::Tokens(t) = self {
            Some(t)
        } else {
            None
        }
    }

    pub fn take_token(self) -> Option<String> {
        if let Self::Token(t) = self {
            Some(t)
        } else {
            None
        }
    }

    pub fn take_tokens(self) -> Option<Vec<String>> {
        if let Self::Tokens(t) = self {
            Some(t)
        } else {
            None
        }
    }
}

impl Serialize for MatchedToken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Token(t) => t.serialize(serializer),
            Self::Tokens(t) => t.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for MatchedToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(MatchedTokensVisitor)
    }
}

struct MatchedTokensVisitor;

impl<'de> Visitor<'de> for MatchedTokensVisitor {
    type Value = MatchedToken;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("string or sequence of strings")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self::Value::Token(v.to_owned()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self::Value::Token(v))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut ret = seq
            .size_hint()
            .map(|n| if n == 0 { 1 } else { n })
            .map(|n| Vec::with_capacity(n))
            .unwrap_or_default();

        while let Some(item) = seq.next_element()? {
            ret.push(item);
        }

        Ok(Self::Value::Tokens(ret))
    }
}
