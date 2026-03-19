use crate::metadata::{
    standard_metadata_tables, MetaColumn, MetaDatasource, MetaExportProfile, MetaImportProfile,
    MetaPolicy, MetaRelation, MetaTable, MetadataTableSchema,
};

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaPlan {
    pub tables: Vec<MetadataTableSchema>,
}

impl SchemaPlan {
    pub fn from_standard_metadata() -> Self {
        Self {
            tables: standard_metadata_tables(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetadataRuntimeModel {
    pub datasources: Vec<MetaDatasource>,
    pub tables: Vec<MetaTable>,
    pub columns: Vec<MetaColumn>,
    pub relations: Vec<MetaRelation>,
    pub policies: Vec<MetaPolicy>,
    pub import_profiles: Vec<MetaImportProfile>,
    pub export_profiles: Vec<MetaExportProfile>,
}

impl MetadataRuntimeModel {
    pub fn new() -> Self {
        Self {
            datasources: Vec::new(),
            tables: Vec::new(),
            columns: Vec::new(),
            relations: Vec::new(),
            policies: Vec::new(),
            import_profiles: Vec::new(),
            export_profiles: Vec::new(),
        }
    }
}
