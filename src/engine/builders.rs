use serde_json::Value;

use super::dialect::SqlDialect;
use super::query::{push_param, BuiltQuery, Pagination, Predicate, Relation, TableRef};

enum SqlValue {
    Param(Value),
    Raw(String),
}

impl SqlValue {
    fn render(self, dialect: &dyn SqlDialect, params: &mut Vec<Value>) -> String {
        match self {
            SqlValue::Param(value) => push_param(dialect, params, value),
            SqlValue::Raw(sql) => sql,
        }
    }
}

pub struct SelectBuilder {
    from: TableRef,
    projections: Vec<String>,
    relations: Vec<Relation>,
    predicates: Vec<Predicate>,
    group_by: Vec<String>,
    order_by: Vec<String>,
    pagination: Option<Pagination>,
}

impl SelectBuilder {
    pub fn new(from: TableRef) -> Self {
        Self {
            from,
            projections: Vec::new(),
            relations: Vec::new(),
            predicates: Vec::new(),
            group_by: Vec::new(),
            order_by: Vec::new(),
            pagination: None,
        }
    }

    pub fn select(mut self, expression: impl Into<String>) -> Self {
        self.projections.push(expression.into());
        self
    }

    pub fn select_as(mut self, expression: impl Into<String>, alias: impl Into<String>) -> Self {
        self.projections
            .push(format!("{} AS {}", expression.into(), alias.into()));
        self
    }

    pub fn relation(mut self, relation: Relation) -> Self {
        self.relations.push(relation);
        self
    }

    pub fn predicate(mut self, predicate: Predicate) -> Self {
        self.predicates.push(predicate);
        self
    }

    pub fn group_by(mut self, expression: impl Into<String>) -> Self {
        self.group_by.push(expression.into());
        self
    }

    pub fn order_by(mut self, expression: impl Into<String>) -> Self {
        self.order_by.push(expression.into());
        self
    }

    pub fn paginate(mut self, pagination: Pagination) -> Self {
        self.pagination = Some(pagination);
        self
    }

    pub fn build(self, dialect: &dyn SqlDialect) -> BuiltQuery {
        let mut params = Vec::new();
        let mut sql = format!(
            "SELECT {} FROM {}",
            if self.projections.is_empty() {
                "*".to_string()
            } else {
                self.projections.join(", ")
            },
            self.from.render()
        );
        if !self.relations.is_empty() {
            sql.push(' ');
            sql.push_str(
                &self
                    .relations
                    .iter()
                    .map(Relation::render)
                    .collect::<Vec<_>>()
                    .join(" "),
            );
        }
        if !self.predicates.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(
                &self
                    .predicates
                    .iter()
                    .map(|predicate| predicate.render(dialect, &mut params))
                    .collect::<Vec<_>>()
                    .join(" AND "),
            );
        }
        if !self.group_by.is_empty() {
            sql.push_str(" GROUP BY ");
            sql.push_str(&self.group_by.join(", "));
        }
        let has_order_by = !self.order_by.is_empty();
        if has_order_by {
            sql.push_str(" ORDER BY ");
            sql.push_str(&self.order_by.join(", "));
        }
        if let Some(pagination) = self.pagination {
            sql = dialect.paginate(sql, &pagination, has_order_by);
        }
        BuiltQuery { sql, params }
    }
}

pub struct InsertBuilder {
    table: String,
    assignments: Vec<(String, SqlValue)>,
}

impl InsertBuilder {
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            assignments: Vec::new(),
        }
    }

    pub fn value(mut self, column: impl Into<String>, value: impl Into<Value>) -> Self {
        self.assignments
            .push((column.into(), SqlValue::Param(value.into())));
        self
    }

    pub fn raw_value(mut self, column: impl Into<String>, sql: impl Into<String>) -> Self {
        self.assignments.push((column.into(), SqlValue::Raw(sql.into())));
        self
    }

    pub fn build(self, dialect: &dyn SqlDialect) -> BuiltQuery {
        let mut params = Vec::new();
        let columns = self
            .assignments
            .iter()
            .map(|(column, _)| column.clone())
            .collect::<Vec<_>>()
            .join(", ");
        let values = self
            .assignments
            .into_iter()
            .map(|(_, value)| value.render(dialect, &mut params))
            .collect::<Vec<_>>()
            .join(", ");
        BuiltQuery {
            sql: format!(
                "INSERT INTO {} ({}) VALUES ({})",
                self.table,
                columns,
                values
            ),
            params,
        }
    }
}

pub struct UpdateBuilder {
    table: String,
    assignments: Vec<(String, SqlValue)>,
    predicates: Vec<Predicate>,
}

impl UpdateBuilder {
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            assignments: Vec::new(),
            predicates: Vec::new(),
        }
    }

    pub fn set(mut self, column: impl Into<String>, value: impl Into<Value>) -> Self {
        self.assignments
            .push((column.into(), SqlValue::Param(value.into())));
        self
    }

    pub fn set_raw(mut self, column: impl Into<String>, sql: impl Into<String>) -> Self {
        self.assignments.push((column.into(), SqlValue::Raw(sql.into())));
        self
    }

    pub fn predicate(mut self, predicate: Predicate) -> Self {
        self.predicates.push(predicate);
        self
    }

    pub fn build(self, dialect: &dyn SqlDialect) -> BuiltQuery {
        let mut params = Vec::new();
        let assignments = self
            .assignments
            .into_iter()
            .map(|(column, value)| format!("{} = {}", column, value.render(dialect, &mut params)))
            .collect::<Vec<_>>()
            .join(", ");
        let mut sql = format!("UPDATE {} SET {}", self.table, assignments);
        if !self.predicates.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(
                &self
                    .predicates
                    .iter()
                    .map(|predicate| predicate.render(dialect, &mut params))
                    .collect::<Vec<_>>()
                    .join(" AND "),
            );
        }
        BuiltQuery { sql, params }
    }
}

pub struct DeleteBuilder {
    table: String,
    predicates: Vec<Predicate>,
}

impl DeleteBuilder {
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            predicates: Vec::new(),
        }
    }

    pub fn predicate(mut self, predicate: Predicate) -> Self {
        self.predicates.push(predicate);
        self
    }

    pub fn build(self, dialect: &dyn SqlDialect) -> BuiltQuery {
        let mut params = Vec::new();
        let mut sql = format!("DELETE FROM {}", self.table);
        if !self.predicates.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(
                &self
                    .predicates
                    .iter()
                    .map(|predicate| predicate.render(dialect, &mut params))
                    .collect::<Vec<_>>()
                    .join(" AND "),
            );
        }
        BuiltQuery { sql, params }
    }
}
