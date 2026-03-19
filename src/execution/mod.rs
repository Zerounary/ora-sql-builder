mod context;
mod datasource;
mod error;
mod executors;
mod helpers;
mod results;

pub use context::{ExecutionContext, ExecutionMode, ExecutionOptions};
pub use datasource::DatasourceManager;
pub use error::ExecutionError;
pub use executors::{
    DeletePlanExecutor, MetadataCatalogExecutor, QueryPlanExecutor, SchemaPlanExecutor,
    WritePlanExecutor,
};
pub use helpers::{schema_plan_from_tables, build_create_queries};
pub use results::{
    DeleteResult, MetadataCatalogResult, QueryResult, SchemaResult, WriteResult,
};

use helpers::{
    build_insert_from_row, dialect_for, execute_non_query, fetch_rows,
    normalize_built_query_for_execution,
};

#[cfg(test)]
mod tests;
