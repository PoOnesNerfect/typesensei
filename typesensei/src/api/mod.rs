use crate::schema::FieldOwned;
use serde::{Deserialize, Serialize};

pub mod collection;
pub mod documents;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionResponse {
    pub name: String,
    pub num_documents: usize,
    pub fields: Vec<FieldOwned>,
    pub default_sorting_field: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResponse {
    pub success: bool,
    pub error: Option<String>,
    pub document: Option<String>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit<T> {
    pub document: T,
    pub highlights: Vec<SearchHighlight>,
    pub text_match: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHighlight {
    pub field: String,
    pub matched_tokens: Vec<String>,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchParams {
    pub collection_name: String,
    pub per_page: usize,
    pub q: String,
}
