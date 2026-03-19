use super::builders::{DeleteBuilder, InsertBuilder, SelectBuilder, UpdateBuilder};
use super::dialect::SqlDialect;
use super::query::BuiltQuery;

#[derive(Default)]
pub struct MetaSqlEngine;

impl MetaSqlEngine {
    pub fn build_select(&self, dialect: &dyn SqlDialect, builder: SelectBuilder) -> BuiltQuery {
        builder.build(dialect)
    }

    pub fn build_insert(&self, dialect: &dyn SqlDialect, builder: InsertBuilder) -> BuiltQuery {
        builder.build(dialect)
    }

    pub fn build_update(&self, dialect: &dyn SqlDialect, builder: UpdateBuilder) -> BuiltQuery {
        builder.build(dialect)
    }

    pub fn build_delete(&self, dialect: &dyn SqlDialect, builder: DeleteBuilder) -> BuiltQuery {
        builder.build(dialect)
    }
}
