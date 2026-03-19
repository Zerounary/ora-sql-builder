use serde_json::Value;

use crate::engine::{ColumnDefinition, CreateTableBuilder, ForeignKeyDefinition};
use crate::sql::StatementType;

pub type MetadataId = i64;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CapabilityMask(Vec<bool>);

impl CapabilityMask {
    pub fn allows(&self, index: usize) -> bool {
        self.0.get(index).copied().unwrap_or(false)
    }
}

impl From<&str> for CapabilityMask {
    fn from(value: &str) -> Self {
        Self(value.chars().map(|ch| ch == '1').collect())
    }
}

impl From<String> for CapabilityMask {
    fn from(value: String) -> Self {
        CapabilityMask::from(value.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum FieldInputKind {
    #[default]
    Text,
    Lookup,
    Ignored,
    Operation,
    Sequence,
    Trigger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetadataFilter {
    Eq(Value),
    Ne(Value),
    Gt(Value),
    Gte(Value),
    Lt(Value),
    Lte(Value),
    Like(String),
    In(Vec<Value>),
    Between { lower: Value, upper: Value },
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetadataFilterExpr {
    Field {
        field: String,
        filter: MetadataFilter,
    },
    Exists {
        sql: String,
        params: Vec<Value>,
    },
    Custom {
        sql: String,
        params: Vec<Value>,
    },
    Raw(String),
    And(Vec<MetadataFilterExpr>),
    Or(Vec<MetadataFilterExpr>),
    Not(Box<MetadataFilterExpr>),
}

impl MetadataFilterExpr {
    pub fn eq(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Field {
            field: field.into(),
            filter: MetadataFilter::Eq(value.into()),
        }
    }

    pub fn ne(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Field {
            field: field.into(),
            filter: MetadataFilter::Ne(value.into()),
        }
    }

    pub fn gt(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Field {
            field: field.into(),
            filter: MetadataFilter::Gt(value.into()),
        }
    }

    pub fn gte(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Field {
            field: field.into(),
            filter: MetadataFilter::Gte(value.into()),
        }
    }

    pub fn lt(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Field {
            field: field.into(),
            filter: MetadataFilter::Lt(value.into()),
        }
    }

    pub fn lte(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Field {
            field: field.into(),
            filter: MetadataFilter::Lte(value.into()),
        }
    }

    pub fn like(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Field {
            field: field.into(),
            filter: MetadataFilter::Like(value.into()),
        }
    }

    pub fn in_list(field: impl Into<String>, values: Vec<Value>) -> Self {
        Self::Field {
            field: field.into(),
            filter: MetadataFilter::In(values),
        }
    }

    pub fn between(
        field: impl Into<String>,
        lower: impl Into<Value>,
        upper: impl Into<Value>,
    ) -> Self {
        Self::Field {
            field: field.into(),
            filter: MetadataFilter::Between {
                lower: lower.into(),
                upper: upper.into(),
            },
        }
    }

    pub fn is_null(field: impl Into<String>) -> Self {
        Self::Field {
            field: field.into(),
            filter: MetadataFilter::IsNull,
        }
    }

    pub fn is_not_null(field: impl Into<String>) -> Self {
        Self::Field {
            field: field.into(),
            filter: MetadataFilter::IsNotNull,
        }
    }

    pub fn exists(sql: impl Into<String>, params: Vec<Value>) -> Self {
        Self::Exists {
            sql: sql.into(),
            params,
        }
    }

    pub fn custom(sql: impl Into<String>, params: Vec<Value>) -> Self {
        Self::Custom {
            sql: sql.into(),
            params,
        }
    }

    pub fn raw(sql: impl Into<String>) -> Self {
        Self::Raw(sql.into())
    }

    pub fn and(filters: Vec<MetadataFilterExpr>) -> Self {
        Self::And(filters)
    }

    pub fn or(filters: Vec<MetadataFilterExpr>) -> Self {
        Self::Or(filters)
    }

    pub fn not(filter: MetadataFilterExpr) -> Self {
        Self::Not(Box::new(filter))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LookupReference {
    pub table: String,
    pub display_column: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkStep {
    pub foreign_key: String,
    pub table: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkReference {
    pub steps: Vec<LinkStep>,
    pub target_column: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldSource {
    Column(String),
    Qualified(String),
    Formula(String),
    Linked(LinkReference),
}

impl Default for FieldSource {
    fn default() -> Self {
        FieldSource::Column(String::new())
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MetadataField {
    pub field_id: MetadataId,
    pub current_table: String,
    pub real_table: Option<String>,
    pub source: FieldSource,
    pub access: CapabilityMask,
    pub nullable: bool,
    pub input_kind: FieldInputKind,
    pub sequence_name: Option<String>,
    pub default_value: Option<Value>,
    pub value: Option<Value>,
    pub sort: Option<SortDirection>,
    pub output_alias: Option<String>,
    pub lookup: Option<LookupReference>,
}

impl MetadataField {
    pub fn new(current_table: impl Into<String>, source: FieldSource) -> Self {
        Self {
            current_table: current_table.into(),
            source,
            ..Default::default()
        }
    }

    pub fn with_real_table(mut self, real_table: impl Into<String>) -> Self {
        self.real_table = Some(real_table.into());
        self
    }

    pub fn with_access(mut self, access: impl Into<CapabilityMask>) -> Self {
        self.access = access.into();
        self
    }

    pub fn with_value(mut self, value: Value) -> Self {
        self.value = Some(value);
        self
    }

    pub fn with_default(mut self, value: Value) -> Self {
        self.default_value = Some(value);
        self
    }

    pub fn with_sort(mut self, sort: SortDirection) -> Self {
        self.sort = Some(sort);
        self
    }

    pub fn with_output_alias(mut self, alias: impl Into<String>) -> Self {
        self.output_alias = Some(alias.into());
        self
    }

    pub fn with_lookup(mut self, table: impl Into<String>, display_column: impl Into<String>) -> Self {
        self.lookup = Some(LookupReference {
            table: table.into(),
            display_column: display_column.into(),
        });
        self.input_kind = FieldInputKind::Lookup;
        self
    }

    pub fn with_sequence(mut self, sequence_name: impl Into<String>) -> Self {
        self.sequence_name = Some(sequence_name.into());
        self.input_kind = FieldInputKind::Sequence;
        self
    }

    pub fn source_column(&self) -> Option<&str> {
        match &self.source {
            FieldSource::Column(column) => Some(column.as_str()),
            _ => None,
        }
    }

    pub fn output_name(&self) -> String {
        if let Some(alias) = &self.output_alias {
            return alias.clone();
        }

        match &self.source {
            FieldSource::Column(column) => column.clone(),
            FieldSource::Qualified(expression) => expression.clone(),
            FieldSource::Formula(expression) => expression.clone(),
            FieldSource::Linked(link) => link.target_column.clone(),
        }
    }
}

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
            .column(
                MetadataColumnSchema::new("enabled", MetadataColumnType::Boolean)
                    .not_null()
                    .default_raw("TRUE"),
            )
            .primary_key(vec!["id"])
            .unique_key(vec!["code"]),
        MetadataTableSchema::new("meta_table")
            .column(MetadataColumnSchema::new("id", MetadataColumnType::BigInt).not_null())
            .column(
                MetadataColumnSchema::new("datasource_id", MetadataColumnType::BigInt).not_null(),
            )
            .column(MetadataColumnSchema::new("table_code", MetadataColumnType::Varchar(64)).not_null())
            .column(MetadataColumnSchema::new("table_name", MetadataColumnType::Varchar(128)).not_null())
            .column(MetadataColumnSchema::new("display_name", MetadataColumnType::Varchar(128)).not_null())
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
            .primary_key(vec!["id"])
            .unique_key(vec!["table_id", "policy_code"])
            .foreign_key(
                MetadataForeignKeySchema::new(vec!["table_id"], "meta_table", vec!["id"])
                    .name("fk_meta_policy_table")
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
