use std::collections::HashMap;
use std::sync::{Arc, Once, RwLock};

use serde_json::{json, Map, Value};
use sqlx::any::{AnyArguments, AnyPoolOptions, AnyRow};
use sqlx::query::Query;
use sqlx::{Any, AnyPool, Column, Executor, Row};

use crate::engine::{BuiltQuery, MetaSqlEngine, SqlDialect, SqliteDialect};
use crate::metadata::{
    DatabaseKind, MetaDatasource, MetadataCatalog, MetadataId, MetadataTableSchema,
};
use crate::metadata_driver::MetadataSqlDriver;
use crate::metadata_mapping::MetadataPersistenceMapper;
use crate::metadata_plan::{DeletePlan, QueryPlan, SchemaPlan, WritePlan};

static ANY_DRIVERS: Once = Once::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionMode {
    Query,
    Write,
    Delete,
    Schema,
    Metadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionOptions {
    pub mode: ExecutionMode,
    pub dry_run: bool,
    pub transactional: bool,
    pub max_rows: Option<usize>,
}

impl Default for ExecutionOptions {
    fn default() -> Self {
        Self {
            mode: ExecutionMode::Query,
            dry_run: false,
            transactional: false,
            max_rows: None,
        }
    }
}

pub struct ExecutionContext<'a> {
    pub manager: &'a DatasourceManager,
    pub datasource: &'a MetaDatasource,
    pub options: ExecutionOptions,
}

impl<'a> ExecutionContext<'a> {
    pub fn new(
        manager: &'a DatasourceManager,
        datasource: &'a MetaDatasource,
        options: ExecutionOptions,
    ) -> Self {
        Self {
            manager,
            datasource,
            options,
        }
    }
}

#[derive(Debug)]
pub enum ExecutionError {
    DatasourceNotFound(MetadataId),
    DatasourceRegistration(String),
    Sqlx(sqlx::Error),
    Planning(String),
    Permission(String),
    Mapping(String),
}

impl From<sqlx::Error> for ExecutionError {
    fn from(value: sqlx::Error) -> Self {
        ExecutionError::Sqlx(value)
    }
}

#[derive(Default, Clone)]
pub struct DatasourceManager {
    pools: Arc<RwLock<HashMap<MetadataId, AnyPool>>>,
}

impl DatasourceManager {
    pub async fn register_datasource(
        &self,
        datasource: &MetaDatasource,
    ) -> Result<(), ExecutionError> {
        ensure_any_drivers();
        let max_connections = datasource
            .options
            .get("max_connections")
            .and_then(Value::as_u64)
            .unwrap_or(1);
        let pool = AnyPoolOptions::new()
            .max_connections(max_connections.try_into().unwrap_or(u32::MAX))
            .connect(&datasource.connection_uri)
            .await
            .map_err(|error| ExecutionError::DatasourceRegistration(error.to_string()))?;
        self.pools
            .write()
            .map_err(|_| {
                ExecutionError::DatasourceRegistration(
                    "failed to acquire datasource registry for writing".to_string(),
                )
            })?
            .insert(datasource.id, pool);
        Ok(())
    }

    pub fn get_pool(&self, datasource_id: MetadataId) -> Result<AnyPool, ExecutionError> {
        self.pools
            .read()
            .map_err(|_| {
                ExecutionError::DatasourceRegistration(
                    "failed to acquire datasource registry for reading".to_string(),
                )
            })?
            .get(&datasource_id)
            .cloned()
            .ok_or(ExecutionError::DatasourceNotFound(datasource_id))
    }

    pub async fn health_check(&self, datasource_id: MetadataId) -> Result<(), ExecutionError> {
        let pool = self.get_pool(datasource_id)?;
        sqlx::query("SELECT 1").execute(&pool).await?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryResult {
    pub sql: String,
    pub rows: Vec<Value>,
    pub row_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteResult {
    pub sql: String,
    pub rows_affected: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteResult {
    pub sql: String,
    pub rows_affected: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaResult {
    pub executed_sql: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataCatalogResult {
    pub inserted_rows: usize,
    pub executed_sql: Vec<String>,
}

pub struct QueryPlanExecutor;
pub struct WritePlanExecutor;
pub struct DeletePlanExecutor;
pub struct SchemaPlanExecutor;
pub struct MetadataCatalogExecutor;

impl QueryPlanExecutor {
    pub fn preview(
        &self,
        datasource: &MetaDatasource,
        plan: &QueryPlan,
    ) -> Result<BuiltQuery, ExecutionError> {
        let dialect = dialect_for(&datasource.database_kind)?;
        Ok(MetadataSqlDriver::new(plan.source_request.clone()).build(dialect.as_ref()))
    }

    pub async fn execute(
        &self,
        context: &ExecutionContext<'_>,
        plan: &QueryPlan,
    ) -> Result<QueryResult, ExecutionError> {
        let built = normalize_built_query_for_execution(
            context.datasource,
            self.preview(context.datasource, plan)?,
        );
        if context.options.dry_run {
            return Ok(QueryResult {
                sql: built.sql,
                rows: Vec::new(),
                row_count: 0,
            });
        }

        let pool = context.manager.get_pool(context.datasource.id)?;
        let mut rows = fetch_rows(&pool, &built).await?;
        if let Some(max_rows) = context.options.max_rows {
            rows.truncate(max_rows);
        }
        let row_count = rows.len();
        Ok(QueryResult {
            sql: built.sql,
            rows,
            row_count,
        })
    }
}

impl WritePlanExecutor {
    pub fn preview(
        &self,
        datasource: &MetaDatasource,
        plan: &WritePlan,
    ) -> Result<BuiltQuery, ExecutionError> {
        let dialect = dialect_for(&datasource.database_kind)?;
        Ok(MetadataSqlDriver::new(plan.source_request.clone()).build(dialect.as_ref()))
    }

    pub async fn execute(
        &self,
        context: &ExecutionContext<'_>,
        plan: &WritePlan,
    ) -> Result<WriteResult, ExecutionError> {
        let built = normalize_built_query_for_execution(
            context.datasource,
            self.preview(context.datasource, plan)?,
        );
        if context.options.dry_run {
            return Ok(WriteResult {
                sql: built.sql,
                rows_affected: 0,
            });
        }

        let pool = context.manager.get_pool(context.datasource.id)?;
        let rows_affected = if context.options.transactional {
            let mut tx = pool.begin().await?;
            let affected = execute_non_query(&mut *tx, &built).await?;
            tx.commit().await?;
            affected
        } else {
            execute_non_query(&pool, &built).await?
        };
        Ok(WriteResult {
            sql: built.sql,
            rows_affected,
        })
    }
}

impl DeletePlanExecutor {
    pub fn preview(
        &self,
        datasource: &MetaDatasource,
        plan: &DeletePlan,
    ) -> Result<BuiltQuery, ExecutionError> {
        let dialect = dialect_for(&datasource.database_kind)?;
        Ok(MetadataSqlDriver::new(plan.source_request.clone()).build(dialect.as_ref()))
    }

    pub async fn execute(
        &self,
        context: &ExecutionContext<'_>,
        plan: &DeletePlan,
    ) -> Result<DeleteResult, ExecutionError> {
        let built = normalize_built_query_for_execution(
            context.datasource,
            self.preview(context.datasource, plan)?,
        );
        if context.options.dry_run {
            return Ok(DeleteResult {
                sql: built.sql,
                rows_affected: 0,
            });
        }

        let pool = context.manager.get_pool(context.datasource.id)?;
        let rows_affected = if context.options.transactional {
            let mut tx = pool.begin().await?;
            let affected = execute_non_query(&mut *tx, &built).await?;
            tx.commit().await?;
            affected
        } else {
            execute_non_query(&pool, &built).await?
        };
        Ok(DeleteResult {
            sql: built.sql,
            rows_affected,
        })
    }
}

impl SchemaPlanExecutor {
    pub fn preview(
        &self,
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

    pub async fn execute(
        &self,
        context: &ExecutionContext<'_>,
        plan: &SchemaPlan,
    ) -> Result<SchemaResult, ExecutionError> {
        let built_queries = self
            .preview(context.datasource, plan)?
            .into_iter()
            .map(|query| normalize_built_query_for_execution(context.datasource, query))
            .collect::<Vec<_>>();
        if context.options.dry_run {
            return Ok(SchemaResult {
                executed_sql: built_queries.into_iter().map(|query| query.sql).collect(),
            });
        }

        let pool = context.manager.get_pool(context.datasource.id)?;
        let mut executed_sql = Vec::with_capacity(built_queries.len());
        if context.options.transactional {
            let mut tx = pool.begin().await?;
            for built in built_queries {
                executed_sql.push(built.sql.clone());
                execute_non_query(&mut *tx, &built).await?;
            }
            tx.commit().await?;
        } else {
            for built in built_queries {
                executed_sql.push(built.sql.clone());
                execute_non_query(&pool, &built).await?;
            }
        }
        Ok(SchemaResult { executed_sql })
    }
}

impl MetadataCatalogExecutor {
    pub fn preview(
        &self,
        datasource: &MetaDatasource,
        catalog: &MetadataCatalog,
    ) -> Result<Vec<BuiltQuery>, ExecutionError> {
        let snapshot = MetadataPersistenceMapper::snapshot_from_catalog(catalog);
        let dialect = dialect_for(&datasource.database_kind)?;
        Ok(snapshot
            .rows
            .iter()
            .map(|row| build_insert_from_row(&row.table, &row.values, dialect.as_ref()))
            .collect())
    }

    pub async fn execute(
        &self,
        context: &ExecutionContext<'_>,
        catalog: &MetadataCatalog,
    ) -> Result<MetadataCatalogResult, ExecutionError> {
        let built_queries = self
            .preview(context.datasource, catalog)?
            .into_iter()
            .map(|query| normalize_built_query_for_execution(context.datasource, query))
            .collect::<Vec<_>>();
        if context.options.dry_run {
            return Ok(MetadataCatalogResult {
                inserted_rows: built_queries.len(),
                executed_sql: built_queries.into_iter().map(|query| query.sql).collect(),
            });
        }

        let pool = context.manager.get_pool(context.datasource.id)?;
        let mut executed_sql = Vec::with_capacity(built_queries.len());
        if context.options.transactional {
            let mut tx = pool.begin().await?;
            for built in &built_queries {
                executed_sql.push(built.sql.clone());
                execute_non_query(&mut *tx, built).await?;
            }
            tx.commit().await?;
        } else {
            for built in &built_queries {
                executed_sql.push(built.sql.clone());
                execute_non_query(&pool, built).await?;
            }
        }
        Ok(MetadataCatalogResult {
            inserted_rows: built_queries.len(),
            executed_sql,
        })
    }
}

fn ensure_any_drivers() {
    ANY_DRIVERS.call_once(sqlx::any::install_default_drivers);
}

fn dialect_for(database_kind: &DatabaseKind) -> Result<Box<dyn SqlDialect>, ExecutionError> {
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

fn build_insert_from_row(
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

fn normalize_built_query_for_execution(
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

async fn execute_non_query<'e, E>(executor: E, built: &BuiltQuery) -> Result<u64, ExecutionError>
where
    E: Executor<'e, Database = Any>,
{
    let result = bind_query(sqlx::query(&built.sql), &built.params)
        .execute(executor)
        .await?;
    Ok(result.rows_affected())
}

async fn fetch_rows<'e, E>(executor: E, built: &BuiltQuery) -> Result<Vec<Value>, ExecutionError>
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::{
        DatabaseKind, MetaColumn, MetaDatasource, MetaExportProfile, MetaImportFieldMapping,
        MetaImportProfile, MetaPolicy, MetaTable, MetadataCatalog, MetadataColumnType,
        MetadataField, MetadataFilterExpr, MetadataQueryOptions, MetadataQueryRequest, PolicyKind,
    };
    use crate::metadata_plan::{DeletePlan, QueryPlan, WritePlan};
    use crate::sql::StatementType;
    use similar_asserts::assert_eq;

    fn sqlite_datasource() -> MetaDatasource {
        MetaDatasource::new(
            1,
            "sqlite_main",
            "SQLite 主数据源",
            DatabaseKind::Sqlite,
            "sqlite::memory:",
        )
        .with_options(json!({"max_connections": 1}))
    }

    #[tokio::test]
    async fn schema_write_query_and_delete_executors_work_with_sqlite() {
        let manager = DatasourceManager::default();
        let datasource = sqlite_datasource();
        manager.register_datasource(&datasource).await.unwrap();
        manager.health_check(datasource.id).await.unwrap();

        let schema_plan = schema_plan_from_tables(vec![
            MetadataTableSchema::new("m_retail")
                .column(crate::metadata::MetadataColumnSchema::new(
                    "id",
                    crate::metadata::MetadataColumnType::BigInt,
                )
                .not_null())
                .column(crate::metadata::MetadataColumnSchema::new(
                    "ad_client_id",
                    crate::metadata::MetadataColumnType::BigInt,
                )
                .not_null())
                .column(crate::metadata::MetadataColumnSchema::new(
                    "ad_org_id",
                    crate::metadata::MetadataColumnType::BigInt,
                )
                .not_null())
                .column(crate::metadata::MetadataColumnSchema::new(
                    "ownerid",
                    crate::metadata::MetadataColumnType::BigInt,
                )
                .not_null())
                .column(crate::metadata::MetadataColumnSchema::new(
                    "modifiered",
                    crate::metadata::MetadataColumnType::BigInt,
                )
                .not_null())
                .column(crate::metadata::MetadataColumnSchema::new(
                    "creationdate",
                    crate::metadata::MetadataColumnType::DateTime,
                )
                .not_null())
                .column(crate::metadata::MetadataColumnSchema::new(
                    "modifieddate",
                    crate::metadata::MetadataColumnType::DateTime,
                )
                .not_null())
                .column(crate::metadata::MetadataColumnSchema::new(
                    "name",
                    crate::metadata::MetadataColumnType::Varchar(64),
                )
                .not_null())
                .column(crate::metadata::MetadataColumnSchema::new(
                    "tenant_id",
                    crate::metadata::MetadataColumnType::BigInt,
                )
                .not_null())
                .primary_key(vec!["id"]),
        ]);
        let schema_executor = SchemaPlanExecutor;
        schema_executor
            .execute(
                &ExecutionContext::new(
                    &manager,
                    &datasource,
                    ExecutionOptions {
                        mode: ExecutionMode::Schema,
                        transactional: true,
                        ..Default::default()
                    },
                ),
                &schema_plan,
            )
            .await
            .unwrap();

        let insert_request = MetadataQueryRequest::new(
            893,
            StatementType::INSERT,
            vec![
                MetadataField::new(
                    "m_retail",
                    crate::metadata::FieldSource::Column("name".to_string()),
                )
                .with_access("1")
                .with_value(json!("苹果")),
                MetadataField::new(
                    "m_retail",
                    crate::metadata::FieldSource::Column("tenant_id".to_string()),
                )
                .with_access("1")
                .with_value(json!(37)),
            ],
        )
        .with_options(MetadataQueryOptions {
            id: Some(1),
            ..Default::default()
        });
        let write_plan = WritePlan::from_insert_request(&insert_request);
        let write_result = WritePlanExecutor
            .execute(
                &ExecutionContext::new(
                    &manager,
                    &datasource,
                    ExecutionOptions {
                        mode: ExecutionMode::Write,
                        transactional: true,
                        ..Default::default()
                    },
                ),
                &write_plan,
            )
            .await
            .unwrap();
        assert_eq!(write_result.rows_affected, 1);

        let query_request = MetadataQueryRequest::new(
            893,
            StatementType::SELECT,
            vec![
                MetadataField::new(
                    "m_retail",
                    crate::metadata::FieldSource::Column("id".to_string()),
                ),
                MetadataField::new(
                    "m_retail",
                    crate::metadata::FieldSource::Column("name".to_string()),
                )
                .with_access("1")
                .with_output_alias("name"),
            ],
        )
        .with_options(MetadataQueryOptions {
            table_filter: Some("tenant_id = 37".to_string()),
            ..Default::default()
        })
        .with_filters(vec![MetadataFilterExpr::eq("name", "苹果")]);
        let query_plan = QueryPlan::from_request(&query_request);
        let query_result = QueryPlanExecutor
            .execute(
                &ExecutionContext::new(
                    &manager,
                    &datasource,
                    ExecutionOptions {
                        mode: ExecutionMode::Query,
                        ..Default::default()
                    },
                ),
                &query_plan,
            )
            .await
            .unwrap();
        assert_eq!(query_result.row_count, 1);
        assert_eq!(query_result.rows[0]["name"], json!("苹果"));

        let delete_request = MetadataQueryRequest::new(
            893,
            StatementType::DELETE,
            vec![MetadataField::new(
                "m_retail",
                crate::metadata::FieldSource::Column("id".to_string()),
            )],
        )
        .with_options(MetadataQueryOptions {
            id: Some(1),
            table_filter: Some("tenant_id = 37".to_string()),
            ..Default::default()
        });
        let delete_plan = DeletePlan::from_request(&delete_request);
        let delete_result = DeletePlanExecutor
            .execute(
                &ExecutionContext::new(
                    &manager,
                    &datasource,
                    ExecutionOptions {
                        mode: ExecutionMode::Delete,
                        transactional: true,
                        ..Default::default()
                    },
                ),
                &delete_plan,
            )
            .await
            .unwrap();
        assert_eq!(delete_result.rows_affected, 1);
    }

    #[tokio::test]
    async fn metadata_catalog_executor_persists_catalog_snapshot() {
        let manager = DatasourceManager::default();
        let datasource = sqlite_datasource();
        manager.register_datasource(&datasource).await.unwrap();

        let schema_executor = SchemaPlanExecutor;
        schema_executor
            .execute(
                &ExecutionContext::new(
                    &manager,
                    &datasource,
                    ExecutionOptions {
                        mode: ExecutionMode::Schema,
                        transactional: true,
                        ..Default::default()
                    },
                ),
                &SchemaPlan::from_standard_metadata(),
            )
            .await
            .unwrap();

        let catalog = MetadataCatalog::new()
            .datasource(sqlite_datasource())
            .table(MetaTable::new(10, 1, "retail", "m_retail", "零售单"))
            .column(MetaColumn::new(
                100,
                10,
                "code",
                "code",
                "单号",
                MetadataColumnType::Varchar(64),
            ))
            .policy(
                MetaPolicy::new(300, 10, "tenant_scope", PolicyKind::RowFilter)
                    .with_filter(MetadataFilterExpr::eq("tenant_id", 37)),
            )
            .import_profile(
                MetaImportProfile::new(400, 10, "retail_import", "零售导入")
                    .field_mapping(MetaImportFieldMapping::new("bill_code", "code").required()),
            )
            .export_profile(
                MetaExportProfile::new(500, 10, "retail_export", "零售导出")
                    .with_selected_columns(vec!["code", "name"])
                    .with_default_filter(MetadataFilterExpr::eq("enabled", true)),
            );

        let result = MetadataCatalogExecutor
            .execute(
                &ExecutionContext::new(
                    &manager,
                    &datasource,
                    ExecutionOptions {
                        mode: ExecutionMode::Metadata,
                        transactional: true,
                        ..Default::default()
                    },
                ),
                &catalog,
            )
            .await
            .unwrap();

        assert!(result.inserted_rows >= 6);
        let pool = manager.get_pool(datasource.id).unwrap();
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM meta_export_profile")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(row.0, 1);
    }
}
