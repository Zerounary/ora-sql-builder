use serde_json::json;
use similar_asserts::assert_eq;

use super::*;
use crate::metadata::{
    DatabaseKind, MetaColumn, MetaDatasource, MetaExportProfile, MetaImportFieldMapping,
    MetaImportProfile, MetaPolicy, MetaTable, MetadataCatalog, MetadataColumnType, MetadataField,
    MetadataFilterExpr, MetadataQueryOptions, MetadataQueryRequest, MetadataTableSchema, PolicyKind,
};
use crate::metadata_plan::{DeletePlan, QueryPlan, SchemaPlan, WritePlan};
use crate::sql::StatementType;

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
