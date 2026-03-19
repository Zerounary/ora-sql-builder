use serde_json::Value;

use super::{LookupReference, MetadataColumnType, MetadataFilterExpr, MetadataId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseKind {
    MySql,
    Postgres,
    Oracle,
    SqlServer,
    Sqlite,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimaryKeyStrategy {
    Manual,
    AutoIncrement,
    Sequence(String),
    Snowflake,
    Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationKind {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyKind {
    RowFilter,
    FieldMask,
    ImportGuard,
    ExportGuard,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaDatasource {
    pub id: MetadataId,
    pub code: String,
    pub name: String,
    pub database_kind: DatabaseKind,
    pub connection_uri: String,
    pub default_schema: Option<String>,
    pub enabled: bool,
    pub options: Value,
}

impl MetaDatasource {
    pub fn new(
        id: MetadataId,
        code: impl Into<String>,
        name: impl Into<String>,
        database_kind: DatabaseKind,
        connection_uri: impl Into<String>,
    ) -> Self {
        Self {
            id,
            code: code.into(),
            name: name.into(),
            database_kind,
            connection_uri: connection_uri.into(),
            default_schema: None,
            enabled: true,
            options: Value::Null,
        }
    }

    pub fn with_default_schema(mut self, schema: impl Into<String>) -> Self {
        self.default_schema = Some(schema.into());
        self
    }

    pub fn with_options(mut self, options: Value) -> Self {
        self.options = options;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaTable {
    pub id: MetadataId,
    pub datasource_id: MetadataId,
    pub table_code: String,
    pub table_name: String,
    pub display_name: String,
    pub primary_key_strategy: PrimaryKeyStrategy,
    pub logical_delete: bool,
    pub audit_enabled: bool,
    pub default_sort: Vec<String>,
    pub enabled: bool,
}

impl MetaTable {
    pub fn new(
        id: MetadataId,
        datasource_id: MetadataId,
        table_code: impl Into<String>,
        table_name: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            id,
            datasource_id,
            table_code: table_code.into(),
            table_name: table_name.into(),
            display_name: display_name.into(),
            primary_key_strategy: PrimaryKeyStrategy::Manual,
            logical_delete: false,
            audit_enabled: false,
            default_sort: Vec::new(),
            enabled: true,
        }
    }

    pub fn with_primary_key_strategy(mut self, strategy: PrimaryKeyStrategy) -> Self {
        self.primary_key_strategy = strategy;
        self
    }

    pub fn with_default_sort<I, S>(mut self, sort: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.default_sort = sort.into_iter().map(Into::into).collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaColumn {
    pub id: MetadataId,
    pub table_id: MetadataId,
    pub column_code: String,
    pub column_name: String,
    pub display_name: String,
    pub column_type: MetadataColumnType,
    pub nullable: bool,
    pub queryable: bool,
    pub editable: bool,
    pub sortable: bool,
    pub primary_key: bool,
    pub default_value_sql: Option<String>,
    pub lookup: Option<LookupReference>,
}

impl MetaColumn {
    pub fn new(
        id: MetadataId,
        table_id: MetadataId,
        column_code: impl Into<String>,
        column_name: impl Into<String>,
        display_name: impl Into<String>,
        column_type: MetadataColumnType,
    ) -> Self {
        Self {
            id,
            table_id,
            column_code: column_code.into(),
            column_name: column_name.into(),
            display_name: display_name.into(),
            column_type,
            nullable: true,
            queryable: true,
            editable: true,
            sortable: true,
            primary_key: false,
            default_value_sql: None,
            lookup: None,
        }
    }

    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    pub fn primary_key(mut self) -> Self {
        self.primary_key = true;
        self
    }

    pub fn with_default_sql(mut self, sql: impl Into<String>) -> Self {
        self.default_value_sql = Some(sql.into());
        self
    }

    pub fn with_lookup(mut self, table: impl Into<String>, display_column: impl Into<String>) -> Self {
        self.lookup = Some(LookupReference {
            table: table.into(),
            display_column: display_column.into(),
        });
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaRelation {
    pub id: MetadataId,
    pub left_table_id: MetadataId,
    pub right_table_id: MetadataId,
    pub relation_kind: RelationKind,
    pub join_type: String,
    pub left_column: String,
    pub right_column: String,
    pub bridge_table: Option<String>,
}

impl MetaRelation {
    pub fn new(
        id: MetadataId,
        left_table_id: MetadataId,
        right_table_id: MetadataId,
        relation_kind: RelationKind,
        left_column: impl Into<String>,
        right_column: impl Into<String>,
    ) -> Self {
        Self {
            id,
            left_table_id,
            right_table_id,
            relation_kind,
            join_type: "LEFT".to_string(),
            left_column: left_column.into(),
            right_column: right_column.into(),
            bridge_table: None,
        }
    }

    pub fn with_join_type(mut self, join_type: impl Into<String>) -> Self {
        self.join_type = join_type.into();
        self
    }

    pub fn with_bridge_table(mut self, bridge_table: impl Into<String>) -> Self {
        self.bridge_table = Some(bridge_table.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetaPolicy {
    pub id: MetadataId,
    pub table_id: MetadataId,
    pub policy_code: String,
    pub policy_kind: PolicyKind,
    pub filter: Option<MetadataFilterExpr>,
    pub enabled: bool,
}

impl MetaPolicy {
    pub fn new(
        id: MetadataId,
        table_id: MetadataId,
        policy_code: impl Into<String>,
        policy_kind: PolicyKind,
    ) -> Self {
        Self {
            id,
            table_id,
            policy_code: policy_code.into(),
            policy_kind,
            filter: None,
            enabled: true,
        }
    }

    pub fn with_filter(mut self, filter: MetadataFilterExpr) -> Self {
        self.filter = Some(filter);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaImportFieldMapping {
    pub source_key: String,
    pub target_column_code: String,
    pub required: bool,
}

impl MetaImportFieldMapping {
    pub fn new(source_key: impl Into<String>, target_column_code: impl Into<String>) -> Self {
        Self {
            source_key: source_key.into(),
            target_column_code: target_column_code.into(),
            required: false,
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaImportProfile {
    pub id: MetadataId,
    pub table_id: MetadataId,
    pub profile_code: String,
    pub display_name: String,
    pub update_keys: Vec<String>,
    pub field_mappings: Vec<MetaImportFieldMapping>,
}

impl MetaImportProfile {
    pub fn new(
        id: MetadataId,
        table_id: MetadataId,
        profile_code: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            id,
            table_id,
            profile_code: profile_code.into(),
            display_name: display_name.into(),
            update_keys: Vec::new(),
            field_mappings: Vec::new(),
        }
    }

    pub fn with_update_keys<I, S>(mut self, keys: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.update_keys = keys.into_iter().map(Into::into).collect();
        self
    }

    pub fn field_mapping(mut self, mapping: MetaImportFieldMapping) -> Self {
        self.field_mappings.push(mapping);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetaExportProfile {
    pub id: MetadataId,
    pub table_id: MetadataId,
    pub profile_code: String,
    pub display_name: String,
    pub selected_column_codes: Vec<String>,
    pub default_filter: Option<MetadataFilterExpr>,
    pub order_by: Vec<String>,
}

impl MetaExportProfile {
    pub fn new(
        id: MetadataId,
        table_id: MetadataId,
        profile_code: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            id,
            table_id,
            profile_code: profile_code.into(),
            display_name: display_name.into(),
            selected_column_codes: Vec::new(),
            default_filter: None,
            order_by: Vec::new(),
        }
    }

    pub fn with_selected_columns<I, S>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.selected_column_codes = columns.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_default_filter(mut self, filter: MetadataFilterExpr) -> Self {
        self.default_filter = Some(filter);
        self
    }

    pub fn with_order_by<I, S>(mut self, order_by: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.order_by = order_by.into_iter().map(Into::into).collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetadataCatalog {
    pub datasources: Vec<MetaDatasource>,
    pub tables: Vec<MetaTable>,
    pub columns: Vec<MetaColumn>,
    pub relations: Vec<MetaRelation>,
    pub policies: Vec<MetaPolicy>,
    pub import_profiles: Vec<MetaImportProfile>,
    pub export_profiles: Vec<MetaExportProfile>,
}

impl MetadataCatalog {
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

    pub fn datasource(mut self, datasource: MetaDatasource) -> Self {
        self.datasources.push(datasource);
        self
    }

    pub fn table(mut self, table: MetaTable) -> Self {
        self.tables.push(table);
        self
    }

    pub fn column(mut self, column: MetaColumn) -> Self {
        self.columns.push(column);
        self
    }

    pub fn relation(mut self, relation: MetaRelation) -> Self {
        self.relations.push(relation);
        self
    }

    pub fn policy(mut self, policy: MetaPolicy) -> Self {
        self.policies.push(policy);
        self
    }

    pub fn import_profile(mut self, profile: MetaImportProfile) -> Self {
        self.import_profiles.push(profile);
        self
    }

    pub fn export_profile(mut self, profile: MetaExportProfile) -> Self {
        self.export_profiles.push(profile);
        self
    }
}
