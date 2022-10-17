use crate::schema::FieldOwned;
use serde::{Deserialize, Serialize};

pub mod collection;
pub mod collections;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionResponse {
    pub name: String,
    pub num_documents: usize,
    pub fields: Vec<FieldOwned>,
    pub default_sorting_field: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentResponse {
    pub success: bool,
    pub error: Option<String>,
    pub document: Option<String>,
}
