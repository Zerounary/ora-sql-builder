use crate::engine::BuiltQuery;
use crate::metadata::{MetaDatasource, MetadataCatalog};
use crate::metadata_driver::MetadataSqlDriver;
use crate::metadata_mapping::MetadataPersistenceMapper;
use crate::metadata_plan::{DeletePlan, QueryPlan, SchemaPlan, WritePlan};

use super::{
    build_create_queries, build_insert_from_row, dialect_for, execute_non_query, fetch_rows,
    normalize_built_query_for_execution, DeleteResult, ExecutionContext, ExecutionError,
    MetadataCatalogResult, QueryResult, SchemaResult, WriteResult,
};

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
        build_create_queries(datasource, plan)
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
