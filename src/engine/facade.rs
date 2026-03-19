use super::builders::{DeleteBuilder, InsertBuilder, SelectBuilder, UpdateBuilder};
use super::ddl::{AlterTableBuilder, CreateTableBuilder, DropTableBuilder};
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

    pub fn build_create_table(
        &self,
        dialect: &dyn SqlDialect,
        builder: CreateTableBuilder,
    ) -> BuiltQuery {
        builder.build(dialect)
    }

    pub fn build_alter_table(
        &self,
        dialect: &dyn SqlDialect,
        builder: AlterTableBuilder,
    ) -> BuiltQuery {
        builder.build(dialect)
    }

    pub fn build_drop_table(
        &self,
        dialect: &dyn SqlDialect,
        builder: DropTableBuilder,
    ) -> BuiltQuery {
        builder.build(dialect)
    }
}
