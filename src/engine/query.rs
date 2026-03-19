use serde_json::Value;

use super::dialect::SqlDialect;

#[derive(Debug, Clone, PartialEq)]
pub struct BuiltQuery {
    pub sql: String,
    pub params: Vec<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pagination {
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRef {
    name: String,
    alias: Option<String>,
}

impl TableRef {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            alias: None,
        }
    }

    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.alias = Some(alias.into());
        self
    }

    pub(crate) fn render(&self) -> String {
        match &self.alias {
            Some(alias) => format!("{} {}", self.name, alias),
            None => self.name.clone(),
        }
    }

    pub(crate) fn alias_or_name(&self) -> &str {
        self.alias.as_deref().unwrap_or(self.name.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
}

impl JoinType {
    pub(crate) fn keyword(&self) -> &str {
        match self {
            JoinType::Inner => "INNER JOIN",
            JoinType::Left => "LEFT JOIN",
            JoinType::Right => "RIGHT JOIN",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relation {
    join_type: JoinType,
    left_alias: String,
    left_column: String,
    right_table: TableRef,
    right_column: String,
}

impl Relation {
    pub fn new(
        join_type: JoinType,
        left_alias: impl Into<String>,
        left_column: impl Into<String>,
        right_table: TableRef,
        right_column: impl Into<String>,
    ) -> Self {
        Self {
            join_type,
            left_alias: left_alias.into(),
            left_column: left_column.into(),
            right_table,
            right_column: right_column.into(),
        }
    }

    pub(crate) fn render(&self) -> String {
        let right_alias = self.right_table.alias_or_name();
        format!(
            "{} {} ON {}.{} = {}.{}",
            self.join_type.keyword(),
            self.right_table.render(),
            self.left_alias,
            self.left_column,
            right_alias,
            self.right_column
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Predicate {
    Eq { field: String, value: Value },
    Like { field: String, value: Value },
    In { field: String, values: Vec<Value> },
    Gte { field: String, value: Value },
    Lte { field: String, value: Value },
    IsNull { field: String },
    Raw(String),
}

impl Predicate {
    pub fn eq(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Eq {
            field: field.into(),
            value: value.into(),
        }
    }

    pub fn like(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Like {
            field: field.into(),
            value: value.into(),
        }
    }

    pub fn in_list(field: impl Into<String>, values: Vec<Value>) -> Self {
        Self::In {
            field: field.into(),
            values,
        }
    }

    pub fn gte(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Gte {
            field: field.into(),
            value: value.into(),
        }
    }

    pub fn lte(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Lte {
            field: field.into(),
            value: value.into(),
        }
    }

    pub fn is_null(field: impl Into<String>) -> Self {
        Self::IsNull {
            field: field.into(),
        }
    }

    pub fn raw(sql: impl Into<String>) -> Self {
        Self::Raw(sql.into())
    }

    pub(crate) fn render(&self, dialect: &dyn SqlDialect, params: &mut Vec<Value>) -> String {
        match self {
            Predicate::Eq { field, value } => {
                format!("{} = {}", field, push_param(dialect, params, value.clone()))
            }
            Predicate::Like { field, value } => {
                format!("{} LIKE {}", field, push_param(dialect, params, value.clone()))
            }
            Predicate::In { field, values } => {
                if values.is_empty() {
                    return "1 = 0".to_string();
                }
                let placeholders = values
                    .iter()
                    .cloned()
                    .map(|value| push_param(dialect, params, value))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} IN ({})", field, placeholders)
            }
            Predicate::Gte { field, value } => {
                format!("{} >= {}", field, push_param(dialect, params, value.clone()))
            }
            Predicate::Lte { field, value } => {
                format!("{} <= {}", field, push_param(dialect, params, value.clone()))
            }
            Predicate::IsNull { field } => format!("{} IS NULL", field),
            Predicate::Raw(sql) => sql.clone(),
        }
    }
}

pub(crate) fn push_param(dialect: &dyn SqlDialect, params: &mut Vec<Value>, value: Value) -> String {
    params.push(value);
    dialect.placeholder(params.len())
}
