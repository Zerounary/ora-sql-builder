use serde_json::{json, Map, Value};
use sqlx::any::{AnyArguments, AnyRow};
use sqlx::query::Query;
use sqlx::{Any, Column, Executor, Row};

use crate::engine::{BuiltQuery, MetaSqlEngine, SqlDialect, SqliteDialect};
use crate::metadata::{DatabaseKind, MetaDatasource, MetadataTableSchema};
use crate::metadata_plan::SchemaPlan;

use super::ExecutionError;

pub fn dialect_for(database_kind: &DatabaseKind) -> Result<Box<dyn SqlDialect>, ExecutionError> {
    match database_kind {
        DatabaseKind::Sqlite => Ok(Box::new(SqliteDialect)),
        DatabaseKind::Postgres => Ok(Box::new(crate::engine::PostgresDialect)),
        DatabaseKind::MySql => Ok(Box::new(crate::engine::MySqlDialect)),
        DatabaseKind::Oracle => Ok(Box::new(crate::engine::OracleDialect)),
        DatabaseKind::SqlServer => Ok(Box::new(crate::engine::SqlServerDialect)),
        DatabaseKind::Custom(kind) => Err(ExecutionError::Planning(format!(
            "custom datasource kind '{}' requires a custom dialect binding",
            kind
        ))),
    }
}

pub fn build_insert_from_row(
    table: &str,
    values: &[(String, Value)],
    dialect: &dyn SqlDialect,
) -> BuiltQuery {
    let mut builder = crate::engine::InsertBuilder::new(table.to_string());
    for (column, value) in values {
        builder = builder.value(column.clone(), value.clone());
    }
    builder.build(dialect)
}

pub fn normalize_built_query_for_execution(
    datasource: &MetaDatasource,
    mut built: BuiltQuery,
) -> BuiltQuery {
    if matches!(datasource.database_kind, DatabaseKind::Sqlite) {
        built.sql = built.sql.replace("sysdate", "CURRENT_TIMESTAMP");
    }
    built
}

fn bind_query<'q>(
    mut query: Query<'q, Any, AnyArguments<'q>>,
    params: &'q [Value],
) -> Query<'q, Any, AnyArguments<'q>> {
    for value in params {
        query = match value {
            Value::Null => query.bind(Option::<String>::None),
            Value::Bool(value) => query.bind(*value),
            Value::Number(number) => {
                if let Some(value) = number.as_i64() {
                    query.bind(value)
                } else if let Some(value) = number.as_u64() {
                    if let Ok(value) = i64::try_from(value) {
                        query.bind(value)
                    } else {
                        query.bind(value as f64)
                    }
                } else if let Some(value) = number.as_f64() {
                    query.bind(value)
                } else {
                    query.bind(number.to_string())
                }
            }
            Value::String(value) => query.bind(value.clone()),
            Value::Array(value) => query.bind(Value::Array(value.clone()).to_string()),
            Value::Object(value) => query.bind(Value::Object(value.clone()).to_string()),
        };
    }
    query
}

pub async fn execute_non_query<'e, E>(executor: E, built: &BuiltQuery) -> Result<u64, ExecutionError>
where
    E: Executor<'e, Database = Any>,
{
    let result = bind_query(sqlx::query(&built.sql), &built.params)
        .execute(executor)
        .await?;
    Ok(result.rows_affected())
}

pub async fn fetch_rows<'e, E>(executor: E, built: &BuiltQuery) -> Result<Vec<Value>, ExecutionError>
where
    E: Executor<'e, Database = Any>,
{
    let rows = bind_query(sqlx::query(&built.sql), &built.params)
        .fetch_all(executor)
        .await?;
    Ok(rows.iter().map(row_to_json).collect())
}

fn row_to_json(row: &AnyRow) -> Value {
    let mut object = Map::new();
    for column in row.columns() {
        object.insert(column.name().to_string(), value_from_row(row, column.name()));
    }
    Value::Object(object)
}

fn value_from_row(row: &AnyRow, name: &str) -> Value {
    if let Ok(value) = row.try_get::<Option<bool>, _>(name) {
        return value.map(Value::Bool).unwrap_or(Value::Null);
    }
    if let Ok(value) = row.try_get::<Option<i64>, _>(name) {
        return value.map(|value| json!(value)).unwrap_or(Value::Null);
    }
    if let Ok(value) = row.try_get::<Option<f64>, _>(name) {
        return value.map(|value| json!(value)).unwrap_or(Value::Null);
    }
    if let Ok(value) = row.try_get::<Option<String>, _>(name) {
        return value.map(Value::String).unwrap_or(Value::Null);
    }
    if let Ok(value) = row.try_get::<Option<Vec<u8>>, _>(name) {
        return value
            .map(|value| Value::Array(value.into_iter().map(|byte| json!(byte)).collect()))
            .unwrap_or(Value::Null);
    }
    Value::Null
}

pub fn schema_plan_from_tables(tables: Vec<MetadataTableSchema>) -> SchemaPlan {
    SchemaPlan { tables }
}

pub fn build_create_queries(
    datasource: &MetaDatasource,
    plan: &SchemaPlan,
) -> Result<Vec<BuiltQuery>, ExecutionError> {
    let engine = MetaSqlEngine::default();
    let dialect = dialect_for(&datasource.database_kind)?;
    Ok(plan
        .tables
        .iter()
        .map(|table| {
            engine.build_create_table(dialect.as_ref(), table.to_create_table_builder().if_not_exists())
        })
        .collect())
}
