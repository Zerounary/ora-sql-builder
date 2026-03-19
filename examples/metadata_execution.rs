use ora_sql_builder::execution::{
    schema_plan_from_tables, DatasourceManager, ExecutionContext, ExecutionMode, ExecutionOptions,
    QueryPlanExecutor, SchemaPlanExecutor, WritePlanExecutor,
};
use ora_sql_builder::metadata::{
    DatabaseKind, MetaDatasource, MetadataColumnSchema, MetadataColumnType, MetadataField,
    MetadataFilterExpr, MetadataQueryOptions, MetadataQueryRequest, MetadataTableSchema,
};
use ora_sql_builder::metadata_plan::{QueryPlan, WritePlan};
use ora_sql_builder::sql::StatementType;
use serde_json::json;

#[tokio::main]
async fn main() {
    let datasource = MetaDatasource::new(
        1,
        "sqlite_main",
        "SQLite 主数据源",
        DatabaseKind::Sqlite,
        "sqlite::memory:",
    )
    .with_options(json!({"max_connections": 1}));
    let manager = DatasourceManager::default();
    manager.register_datasource(&datasource).await.unwrap();

    let schema_plan = schema_plan_from_tables(vec![
        MetadataTableSchema::new("m_demo")
            .column(MetadataColumnSchema::new("id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("ad_client_id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("ad_org_id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("ownerid", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("modifiered", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("creationdate", MetadataColumnType::DateTime).not_null())
            .column(MetadataColumnSchema::new("modifieddate", MetadataColumnType::DateTime).not_null())
            .column(MetadataColumnSchema::new("name", MetadataColumnType::Varchar(64)).not_null())
            .primary_key(vec!["id"]),
    ]);
    SchemaPlanExecutor
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

    let write_plan = WritePlan::from_insert_request(
        &MetadataQueryRequest::new(
            893,
            StatementType::INSERT,
            vec![
                MetadataField::new("m_demo", ora_sql_builder::metadata::FieldSource::Column("name".to_string()))
                    .with_access("1")
                    .with_value(json!("执行层示例")),
            ],
        )
        .with_options(MetadataQueryOptions {
            id: Some(1),
            ..Default::default()
        }),
    );
    WritePlanExecutor
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

    let query_plan = QueryPlan::from_request(
        &MetadataQueryRequest::new(
            893,
            StatementType::SELECT,
            vec![
                MetadataField::new("m_demo", ora_sql_builder::metadata::FieldSource::Column("id".to_string())),
                MetadataField::new("m_demo", ora_sql_builder::metadata::FieldSource::Column("name".to_string()))
                    .with_access("1")
                    .with_output_alias("name"),
            ],
        )
        .with_options(MetadataQueryOptions::default())
        .with_filters(vec![MetadataFilterExpr::eq("name", "执行层示例")]),
    );
    let result = QueryPlanExecutor
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

    println!("Execution Query SQL:\n{}\n", result.sql);
    println!("Execution Query Rows:\n{:#?}\n", result.rows);
}
