use crate::engine::{ColumnDefinition, CreateTableBuilder, ForeignKeyDefinition};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataColumnType {
    BigInt,
    Integer,
    Boolean,
    Text,
    Json,
    DateTime,
    Decimal { precision: u8, scale: u8 },
    Varchar(usize),
}

impl MetadataColumnType {
    pub fn sql_type(&self) -> String {
        match self {
            MetadataColumnType::BigInt => "BIGINT".to_string(),
            MetadataColumnType::Integer => "INTEGER".to_string(),
            MetadataColumnType::Boolean => "BOOLEAN".to_string(),
            MetadataColumnType::Text => "TEXT".to_string(),
            MetadataColumnType::Json => "JSON".to_string(),
            MetadataColumnType::DateTime => "TIMESTAMP".to_string(),
            MetadataColumnType::Decimal { precision, scale } => {
                format!("DECIMAL({}, {})", precision, scale)
            }
            MetadataColumnType::Varchar(length) => format!("VARCHAR({})", length),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataColumnSchema {
    pub name: String,
    pub column_type: MetadataColumnType,
    pub nullable: bool,
    pub unique: bool,
    pub default_raw: Option<String>,
}

impl MetadataColumnSchema {
    pub fn new(name: impl Into<String>, column_type: MetadataColumnType) -> Self {
        Self {
            name: name.into(),
            column_type,
            nullable: true,
            unique: false,
            default_raw: None,
        }
    }

    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    pub fn default_raw(mut self, sql: impl Into<String>) -> Self {
        self.default_raw = Some(sql.into());
        self
    }

    pub fn to_column_definition(&self) -> ColumnDefinition {
        let mut column = ColumnDefinition::new(self.name.clone(), self.column_type.sql_type());
        if !self.nullable {
            column = column.not_null();
        }
        if self.unique {
            column = column.unique();
        }
        if let Some(default_raw) = &self.default_raw {
            column = column.default_raw(default_raw.clone());
        }
        column
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataForeignKeySchema {
    pub name: Option<String>,
    pub columns: Vec<String>,
    pub referenced_table: String,
    pub referenced_columns: Vec<String>,
    pub on_delete: Option<String>,
}

impl MetadataForeignKeySchema {
    pub fn new<I, S, J, T>(
        columns: I,
        referenced_table: impl Into<String>,
        referenced_columns: J,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
        J: IntoIterator<Item = T>,
        T: Into<String>,
    {
        Self {
            name: None,
            columns: columns.into_iter().map(Into::into).collect(),
            referenced_table: referenced_table.into(),
            referenced_columns: referenced_columns.into_iter().map(Into::into).collect(),
            on_delete: None,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn on_delete(mut self, action: impl Into<String>) -> Self {
        self.on_delete = Some(action.into());
        self
    }

    pub fn to_foreign_key_definition(&self) -> ForeignKeyDefinition {
        let mut definition = ForeignKeyDefinition::new(
            self.columns.clone(),
            self.referenced_table.clone(),
            self.referenced_columns.clone(),
        );
        if let Some(name) = &self.name {
            definition = definition.name(name.clone());
        }
        if let Some(on_delete) = &self.on_delete {
            definition = definition.on_delete(on_delete.clone());
        }
        definition
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataTableSchema {
    pub name: String,
    pub columns: Vec<MetadataColumnSchema>,
    pub primary_key: Vec<String>,
    pub unique_keys: Vec<Vec<String>>,
    pub foreign_keys: Vec<MetadataForeignKeySchema>,
}

impl MetadataTableSchema {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            columns: Vec::new(),
            primary_key: Vec::new(),
            unique_keys: Vec::new(),
            foreign_keys: Vec::new(),
        }
    }

    pub fn column(mut self, column: MetadataColumnSchema) -> Self {
        self.columns.push(column);
        self
    }

    pub fn primary_key<I, S>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.primary_key = columns.into_iter().map(Into::into).collect();
        self
    }

    pub fn unique_key<I, S>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.unique_keys
            .push(columns.into_iter().map(Into::into).collect());
        self
    }

    pub fn foreign_key(mut self, foreign_key: MetadataForeignKeySchema) -> Self {
        self.foreign_keys.push(foreign_key);
        self
    }

    pub fn to_create_table_builder(&self) -> CreateTableBuilder {
        let mut builder = CreateTableBuilder::new(self.name.clone());
        for column in &self.columns {
            builder = builder.column(column.to_column_definition());
        }
        if !self.primary_key.is_empty() {
            builder = builder.primary_key(self.primary_key.clone());
        }
        for unique_key in &self.unique_keys {
            builder = builder.unique(unique_key.clone());
        }
        for foreign_key in &self.foreign_keys {
            builder = builder.foreign_key(foreign_key.to_foreign_key_definition());
        }
        builder
    }
}

pub fn standard_metadata_tables() -> Vec<MetadataTableSchema> {
    vec![
        MetadataTableSchema::new("meta_datasource")
            .column(MetadataColumnSchema::new("id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("code", MetadataColumnType::Varchar(64)).not_null())
            .column(MetadataColumnSchema::new("name", MetadataColumnType::Varchar(128)).not_null())
            .column(MetadataColumnSchema::new("db_type", MetadataColumnType::Varchar(32)).not_null())
            .column(MetadataColumnSchema::new("dsn", MetadataColumnType::Text).not_null())
            .column(MetadataColumnSchema::new("default_schema", MetadataColumnType::Varchar(64)))
            .column(MetadataColumnSchema::new("options", MetadataColumnType::Json).not_null())
            .column(
                MetadataColumnSchema::new("enabled", MetadataColumnType::Boolean)
                    .not_null()
                    .default_raw("TRUE"),
            )
            .primary_key(vec!["id"])
            .unique_key(vec!["code"]),
        MetadataTableSchema::new("meta_table")
            .column(MetadataColumnSchema::new("id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("datasource_id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("table_code", MetadataColumnType::Varchar(64)).not_null())
            .column(MetadataColumnSchema::new("table_name", MetadataColumnType::Varchar(128)).not_null())
            .column(MetadataColumnSchema::new("display_name", MetadataColumnType::Varchar(128)).not_null())
            .column(MetadataColumnSchema::new("primary_key_strategy", MetadataColumnType::Varchar(64)).not_null())
            .column(MetadataColumnSchema::new("logical_delete", MetadataColumnType::Boolean).not_null())
            .column(MetadataColumnSchema::new("audit_enabled", MetadataColumnType::Boolean).not_null())
            .column(MetadataColumnSchema::new("default_sort", MetadataColumnType::Json).not_null())
            .column(
                MetadataColumnSchema::new("enabled", MetadataColumnType::Boolean)
                    .not_null()
                    .default_raw("TRUE"),
            )
            .primary_key(vec!["id"])
            .unique_key(vec!["datasource_id", "table_code"])
            .foreign_key(
                MetadataForeignKeySchema::new(vec!["datasource_id"], "meta_datasource", vec!["id"])
                    .name("fk_meta_table_datasource")
                    .on_delete("CASCADE"),
            ),
        MetadataTableSchema::new("meta_column")
            .column(MetadataColumnSchema::new("id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("table_id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("column_code", MetadataColumnType::Varchar(64)).not_null())
            .column(MetadataColumnSchema::new("column_name", MetadataColumnType::Varchar(128)).not_null())
            .column(MetadataColumnSchema::new("display_name", MetadataColumnType::Varchar(128)).not_null())
            .column(MetadataColumnSchema::new("data_type", MetadataColumnType::Varchar(32)).not_null())
            .column(MetadataColumnSchema::new("nullable", MetadataColumnType::Boolean).not_null())
            .column(MetadataColumnSchema::new("queryable", MetadataColumnType::Boolean).not_null())
            .column(MetadataColumnSchema::new("editable", MetadataColumnType::Boolean).not_null())
            .column(MetadataColumnSchema::new("sortable", MetadataColumnType::Boolean).not_null())
            .column(MetadataColumnSchema::new("primary_key", MetadataColumnType::Boolean).not_null())
            .column(MetadataColumnSchema::new("default_value_sql", MetadataColumnType::Text))
            .primary_key(vec!["id"])
            .unique_key(vec!["table_id", "column_code"])
            .foreign_key(
                MetadataForeignKeySchema::new(vec!["table_id"], "meta_table", vec!["id"])
                    .name("fk_meta_column_table")
                    .on_delete("CASCADE"),
            ),
        MetadataTableSchema::new("meta_relation")
            .column(MetadataColumnSchema::new("id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("left_table_id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("right_table_id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("relation_type", MetadataColumnType::Varchar(32)).not_null())
            .column(MetadataColumnSchema::new("join_type", MetadataColumnType::Varchar(16)).not_null())
            .column(MetadataColumnSchema::new("left_column", MetadataColumnType::Varchar(64)).not_null())
            .column(MetadataColumnSchema::new("right_column", MetadataColumnType::Varchar(64)).not_null())
            .column(MetadataColumnSchema::new("bridge_table", MetadataColumnType::Varchar(128)))
            .primary_key(vec!["id"])
            .foreign_key(
                MetadataForeignKeySchema::new(vec!["left_table_id"], "meta_table", vec!["id"])
                    .name("fk_meta_relation_left_table")
                    .on_delete("CASCADE"),
            )
            .foreign_key(
                MetadataForeignKeySchema::new(vec!["right_table_id"], "meta_table", vec!["id"])
                    .name("fk_meta_relation_right_table")
                    .on_delete("CASCADE"),
            ),
        MetadataTableSchema::new("meta_policy")
            .column(MetadataColumnSchema::new("id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("table_id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("policy_code", MetadataColumnType::Varchar(64)).not_null())
            .column(MetadataColumnSchema::new("policy_type", MetadataColumnType::Varchar(32)).not_null())
            .column(MetadataColumnSchema::new("policy_expr", MetadataColumnType::Text).not_null())
            .column(MetadataColumnSchema::new("enabled", MetadataColumnType::Boolean).not_null())
            .primary_key(vec!["id"])
            .unique_key(vec!["table_id", "policy_code"])
            .foreign_key(
                MetadataForeignKeySchema::new(vec!["table_id"], "meta_table", vec!["id"])
                    .name("fk_meta_policy_table")
                    .on_delete("CASCADE"),
            ),
        MetadataTableSchema::new("meta_import_profile")
            .column(MetadataColumnSchema::new("id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("table_id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("profile_code", MetadataColumnType::Varchar(64)).not_null())
            .column(MetadataColumnSchema::new("display_name", MetadataColumnType::Varchar(128)).not_null())
            .column(MetadataColumnSchema::new("update_keys", MetadataColumnType::Json).not_null())
            .primary_key(vec!["id"])
            .unique_key(vec!["table_id", "profile_code"])
            .foreign_key(
                MetadataForeignKeySchema::new(vec!["table_id"], "meta_table", vec!["id"])
                    .name("fk_meta_import_profile_table")
                    .on_delete("CASCADE"),
            ),
        MetadataTableSchema::new("meta_import_mapping")
            .column(MetadataColumnSchema::new("id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("profile_id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("source_key", MetadataColumnType::Varchar(128)).not_null())
            .column(MetadataColumnSchema::new("target_column_code", MetadataColumnType::Varchar(64)).not_null())
            .column(MetadataColumnSchema::new("required", MetadataColumnType::Boolean).not_null())
            .primary_key(vec!["id"])
            .foreign_key(
                MetadataForeignKeySchema::new(vec!["profile_id"], "meta_import_profile", vec!["id"])
                    .name("fk_meta_import_mapping_profile")
                    .on_delete("CASCADE"),
            ),
        MetadataTableSchema::new("meta_export_profile")
            .column(MetadataColumnSchema::new("id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("table_id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("profile_code", MetadataColumnType::Varchar(64)).not_null())
            .column(MetadataColumnSchema::new("display_name", MetadataColumnType::Varchar(128)).not_null())
            .column(MetadataColumnSchema::new("selected_columns", MetadataColumnType::Json).not_null())
            .column(MetadataColumnSchema::new("default_filter", MetadataColumnType::Text))
            .column(MetadataColumnSchema::new("order_by", MetadataColumnType::Json).not_null())
            .primary_key(vec!["id"])
            .unique_key(vec!["table_id", "profile_code"])
            .foreign_key(
                MetadataForeignKeySchema::new(vec!["table_id"], "meta_table", vec!["id"])
                    .name("fk_meta_export_profile_table")
                    .on_delete("CASCADE"),
            ),
        MetadataTableSchema::new("meta_operation_log")
            .column(MetadataColumnSchema::new("id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("operator_id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("target_table", MetadataColumnType::Varchar(128)).not_null())
            .column(MetadataColumnSchema::new("statement_type", MetadataColumnType::Varchar(16)).not_null())
            .column(MetadataColumnSchema::new("payload", MetadataColumnType::Json).not_null())
            .column(
                MetadataColumnSchema::new("created_at", MetadataColumnType::DateTime)
                    .not_null()
                    .default_raw("CURRENT_TIMESTAMP"),
            )
            .primary_key(vec!["id"]),
    ]
}
