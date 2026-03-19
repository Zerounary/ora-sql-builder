use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct QueryResult {
    pub sql: String,
    pub rows: Vec<Value>,
    pub row_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteResult {
    pub sql: String,
    pub rows_affected: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteResult {
    pub sql: String,
    pub rows_affected: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaResult {
    pub executed_sql: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataCatalogResult {
    pub inserted_rows: usize,
    pub executed_sql: Vec<String>,
}
