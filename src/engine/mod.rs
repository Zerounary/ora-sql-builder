mod builders;
mod ddl;
mod dialect;
mod facade;
mod query;

pub use builders::{DeleteBuilder, InsertBuilder, SelectBuilder, UpdateBuilder};
pub use ddl::{
    AlterTableBuilder, ColumnDefinition, CreateTableBuilder, DdlValue, DropTableBuilder,
    ForeignKeyDefinition,
};
pub use dialect::{
    MySqlDialect, OracleDialect, PostgresDialect, SqlDialect, SqlServerDialect, SqliteDialect,
};
pub use facade::MetaSqlEngine;
pub use query::{BuiltQuery, JoinType, Pagination, Predicate, Relation, TableRef};

#[cfg(test)]
mod tests;
