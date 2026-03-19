use serde_json::Value;

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
}

impl MetadataQueryRequest {
    pub fn new(user_id: MetadataId, statement_type: StatementType, fields: Vec<MetadataField>) -> Self {
        Self {
            user_id,
            statement_type,
            fields,
            options: MetadataQueryOptions::default(),
        }
    }

    pub fn with_options(mut self, options: MetadataQueryOptions) -> Self {
        self.options = options;
        self
    }
}
