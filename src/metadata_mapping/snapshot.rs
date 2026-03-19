use serde_json::Value;

use crate::engine::CreateTableBuilder;
use crate::metadata::MetadataTableSchema;

#[derive(Debug, Clone, PartialEq)]
pub struct MetadataPersistenceRow {
    pub table: String,
    pub values: Vec<(String, Value)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetadataPersistenceSnapshot {
    pub schemas: Vec<MetadataTableSchema>,
    pub rows: Vec<MetadataPersistenceRow>,
}

impl MetadataPersistenceSnapshot {
    pub fn rows_for(&self, table: &str) -> Vec<&MetadataPersistenceRow> {
        self.rows.iter().filter(|row| row.table == table).collect()
    }

    pub fn ddl_builders(&self) -> Vec<CreateTableBuilder> {
        self.schemas
            .iter()
            .map(|schema| schema.to_create_table_builder())
            .collect()
    }
}
