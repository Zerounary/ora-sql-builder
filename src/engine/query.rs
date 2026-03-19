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
    Ne { field: String, value: Value },
    Gt { field: String, value: Value },
    Like { field: String, value: Value },
    In { field: String, values: Vec<Value> },
    Gte { field: String, value: Value },
    Lt { field: String, value: Value },
    Lte { field: String, value: Value },
    Between {
        field: String,
        lower: Value,
        upper: Value,
    },
    IsNull { field: String },
    IsNotNull { field: String },
    Exists { sql: String, params: Vec<Value> },
    Custom { sql: String, params: Vec<Value> },
    And(Vec<Predicate>),
    Or(Vec<Predicate>),
    Not(Box<Predicate>),
    Raw(String),
}

impl Predicate {
    pub fn eq(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Eq {
            field: field.into(),
            value: value.into(),
        }
    }

    pub fn ne(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Ne {
            field: field.into(),
            value: value.into(),
        }
    }

    pub fn gt(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Gt {
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

    pub fn lt(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Lt {
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

    pub fn between(
        field: impl Into<String>,
        lower: impl Into<Value>,
        upper: impl Into<Value>,
    ) -> Self {
        Self::Between {
            field: field.into(),
            lower: lower.into(),
            upper: upper.into(),
        }
    }

    pub fn is_null(field: impl Into<String>) -> Self {
        Self::IsNull {
            field: field.into(),
        }
    }

    pub fn is_not_null(field: impl Into<String>) -> Self {
        Self::IsNotNull {
            field: field.into(),
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

    pub fn and(predicates: Vec<Predicate>) -> Self {
        Self::And(predicates)
    }

    pub fn or(predicates: Vec<Predicate>) -> Self {
        Self::Or(predicates)
    }

    pub fn not(predicate: Predicate) -> Self {
        Self::Not(Box::new(predicate))
    }

    pub fn raw(sql: impl Into<String>) -> Self {
        Self::Raw(sql.into())
    }

    pub(crate) fn render(&self, dialect: &dyn SqlDialect, params: &mut Vec<Value>) -> String {
        match self {
            Predicate::Eq { field, value } => {
                format!("{} = {}", field, push_param(dialect, params, value.clone()))
            }
            Predicate::Ne { field, value } => {
                format!("{} != {}", field, push_param(dialect, params, value.clone()))
            }
            Predicate::Gt { field, value } => {
                format!("{} > {}", field, push_param(dialect, params, value.clone()))
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
            Predicate::Lt { field, value } => {
                format!("{} < {}", field, push_param(dialect, params, value.clone()))
            }
            Predicate::Lte { field, value } => {
                format!("{} <= {}", field, push_param(dialect, params, value.clone()))
            }
            Predicate::Between {
                field,
                lower,
                upper,
            } => format!(
                "{} BETWEEN {} AND {}",
                field,
                push_param(dialect, params, lower.clone()),
                push_param(dialect, params, upper.clone())
            ),
            Predicate::IsNull { field } => format!("{} IS NULL", field),
            Predicate::IsNotNull { field } => format!("{} IS NOT NULL", field),
            Predicate::Exists {
                sql,
                params: custom_params,
            } => format!(
                "EXISTS ({})",
                render_parameterized_sql(sql, custom_params, dialect, params)
            ),
            Predicate::Custom {
                sql,
                params: custom_params,
            } => render_parameterized_sql(sql, custom_params, dialect, params),
            Predicate::And(predicates) => render_group(predicates, "AND", dialect, params),
            Predicate::Or(predicates) => render_group(predicates, "OR", dialect, params),
            Predicate::Not(predicate) => {
                format!("NOT ({})", predicate.render(dialect, params))
            }
            Predicate::Raw(sql) => sql.clone(),
        }
    }
}

fn render_group(
    predicates: &[Predicate],
    operator: &str,
    dialect: &dyn SqlDialect,
    params: &mut Vec<Value>,
) -> String {
    if predicates.is_empty() {
        return match operator {
            "AND" => "1 = 1".to_string(),
            _ => "1 = 0".to_string(),
        };
    }

    format!(
        "({})",
        predicates
            .iter()
            .map(|predicate| predicate.render(dialect, params))
            .collect::<Vec<_>>()
            .join(&format!(" {} ", operator))
    )
}

fn render_parameterized_sql(
    sql: &str,
    custom_params: &[Value],
    dialect: &dyn SqlDialect,
    params: &mut Vec<Value>,
) -> String {
    for value in custom_params.iter().cloned() {
        params.push(value);
    }
    let mut next_index = params.len() - custom_params.len();
    let mut rendered = String::new();
    for ch in sql.chars() {
        if ch == '?' {
            next_index += 1;
            rendered.push_str(&dialect.placeholder(next_index));
        } else {
            rendered.push(ch);
        }
    }
    rendered
}

pub(crate) fn push_param(dialect: &dyn SqlDialect, params: &mut Vec<Value>, value: Value) -> String {
    params.push(value);
    dialect.placeholder(params.len())
}
