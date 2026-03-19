use crate::sql::StatementType;

use super::{MetadataField, MetadataFilterExpr, MetadataId};

#[derive(Debug, Clone, PartialEq)]
pub struct MetadataQueryOptions {
    pub id: Option<MetadataId>,
    pub mask_index: usize,
    pub grouped: bool,
    pub table_filter: Option<String>,
    pub client_id: MetadataId,
    pub org_id: MetadataId,
}

impl Default for MetadataQueryOptions {
    fn default() -> Self {
        Self {
            id: None,
            mask_index: 0,
            grouped: false,
            table_filter: None,
            client_id: 37,
            org_id: 27,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetadataQueryRequest {
    pub user_id: MetadataId,
    pub statement_type: StatementType,
    pub fields: Vec<MetadataField>,
    pub options: MetadataQueryOptions,
    pub filters: Vec<MetadataFilterExpr>,
    pub having: Vec<MetadataFilterExpr>,
}

impl MetadataQueryRequest {
    pub fn new(user_id: MetadataId, statement_type: StatementType, fields: Vec<MetadataField>) -> Self {
        Self {
            user_id,
            statement_type,
            fields,
            options: MetadataQueryOptions::default(),
            filters: Vec::new(),
            having: Vec::new(),
        }
    }

    pub fn with_options(mut self, options: MetadataQueryOptions) -> Self {
        self.options = options;
        self
    }

    pub fn with_filters(mut self, filters: Vec<MetadataFilterExpr>) -> Self {
        self.filters = filters;
        self
    }

    pub fn with_having(mut self, filters: Vec<MetadataFilterExpr>) -> Self {
        self.having = filters;
        self
    }
}
