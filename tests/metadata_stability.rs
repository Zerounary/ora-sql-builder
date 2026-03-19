use std::fs;
use std::path::PathBuf;

use ora_sql_builder::engine::{
    MySqlDialect, OracleDialect, PostgresDialect, SqlDialect, SqlServerDialect, SqliteDialect,
};
use ora_sql_builder::execution::{
    schema_plan_from_tables, DatasourceManager, ExecutionContext, ExecutionMode, ExecutionOptions,
    QueryPlanExecutor, SchemaPlanExecutor, WritePlanExecutor, DeletePlanExecutor,
};
use ora_sql_builder::metadata::DatabaseKind;
use ora_sql_builder::metadata_demo::{
    demo_sales_table_schemas, direct_order_insert_request, imported_order_insert_request,
    order_delete_request, order_export_request, order_query_request, order_update_request,
    sqlite_demo_datasource,
};
use ora_sql_builder::metadata_driver::MetadataSqlDriver;
use ora_sql_builder::metadata_plan::{DeletePlan, QueryPlan, WritePlan};
use serde_json::Value;
use similar_asserts::assert_eq;

#[test]
fn datasource_file_covers_all_supported_databases() {
    let json = read_supported_datasources();
    let datasources = json
        .get("datasources")
        .and_then(Value::as_array)
        .expect("supported_datasources.json must contain datasources array");

    let mut kinds = datasources
        .iter()
        .filter_map(|item| item.get("database_kind").and_then(Value::as_str))
        .map(str::to_string)
        .collect::<Vec<_>>();
    kinds.sort();

    assert_eq!(
        kinds,
        vec![
            "mysql".to_string(),
            "oracle".to_string(),
            "postgres".to_string(),
            "sqlite".to_string(),
            "sqlserver".to_string(),
        ]
    );
}

#[test]
fn metadata_preview_scenarios_cover_all_supported_dialects() {
    let scenarios = vec![
        ("query", order_query_request()),
        ("export", order_export_request()),
        (
            "insert",
            direct_order_insert_request(1, "SO-001", 1, 1, 180.0, "OPEN"),
        ),
        (
            "import",
            imported_order_insert_request(2, "SO-002", "总部店", "张三", 260.0, "APPROVED"),
        ),
        ("update", order_update_request(2, 320.0, "APPROVED")),
        ("delete", order_delete_request(2)),
    ];

    for (database_kind, dialect) in supported_dialects() {
        for (scenario, request) in &scenarios {
            let built = MetadataSqlDriver::new(request.clone()).build(dialect.as_ref());
            assert!(
                !built.sql.trim().is_empty(),
                "{} scenario should build SQL for {:?}",
                scenario,
                database_kind
            );
            if !built.params.is_empty() {
                let expected_placeholder = match database_kind {
                    DatabaseKind::MySql | DatabaseKind::Sqlite => Some("?"),
                    DatabaseKind::Postgres => Some("$"),
                    DatabaseKind::Oracle => Some(":"),
                    DatabaseKind::SqlServer => Some("@p"),
                    DatabaseKind::Custom(_) => None,
                };
                if let Some(expected_placeholder) = expected_placeholder {
                    assert!(
                        built.sql.contains(expected_placeholder),
                        "{} scenario should preserve placeholder style for {:?}",
                        scenario,
                        database_kind
                    );
                }
            }
        }
    }
}

#[tokio::test]
async fn sqlite_execution_flow_covers_query_filter_relation_create_import_update_delete_export() {
    let manager = DatasourceManager::default();
    let datasource = sqlite_demo_datasource();
    manager.register_datasource(&datasource).await.unwrap();

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
            &schema_plan_from_tables(demo_sales_table_schemas()),
        )
        .await
        .unwrap();

    let pool = manager.get_pool(datasource.id).unwrap();
    sqlx::query("INSERT INTO demo_store (id, name, region) VALUES (1, '总部店', '华东'), (2, '分部店', '华南')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO demo_customer (id, name, level, enabled) VALUES (1, '张三', 'VIP', 'Y'), (2, '李四', 'NORMAL', 'Y')")
        .execute(&pool)
        .await
        .unwrap();

    let direct_write = WritePlan::from_insert_request(&direct_order_insert_request(
        1,
        "SO-001",
        1,
        1,
        180.0,
        "OPEN",
    ));
    let direct_result = WritePlanExecutor
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
            &direct_write,
        )
        .await
        .unwrap();
    assert_eq!(direct_result.rows_affected, 1);

    let import_write = WritePlan::from_insert_request(&imported_order_insert_request(
        2,
        "SO-002",
        "总部店",
        "张三",
        260.0,
        "APPROVED",
    ));
    let import_result = WritePlanExecutor
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
            &import_write,
        )
        .await
        .unwrap();
    assert_eq!(import_result.rows_affected, 1);

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
            &QueryPlan::from_request(&order_query_request()),
        )
        .await
        .unwrap();
    assert_eq!(query_result.row_count, 2);
    assert_eq!(query_result.rows[0]["store_name"], Value::String("总部店".to_string()));
    assert_eq!(query_result.rows[0]["customer_name"], Value::String("张三".to_string()));

    let update_result = WritePlanExecutor
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
            &WritePlan::from_update_request(&order_update_request(2, 320.0, "APPROVED")),
        )
        .await
        .unwrap();
    assert_eq!(update_result.rows_affected, 1);

    let export_result = QueryPlanExecutor
        .execute(
            &ExecutionContext::new(
                &manager,
                &datasource,
                ExecutionOptions {
                    mode: ExecutionMode::Query,
                    ..Default::default()
                },
            ),
            &QueryPlan::from_request(&order_export_request()),
        )
        .await
        .unwrap();
    assert_eq!(export_result.row_count, 2);
    assert_eq!(export_result.rows[1]["order_code"], Value::String("SO-002".to_string()));

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
            &DeletePlan::from_request(&order_delete_request(1)),
        )
        .await
        .unwrap();
    assert_eq!(delete_result.rows_affected, 1);

    let remaining: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM demo_sale_order")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(remaining.0, 1);
}

fn supported_dialects() -> Vec<(DatabaseKind, Box<dyn SqlDialect>)> {
    vec![
        (DatabaseKind::MySql, Box::new(MySqlDialect)),
        (DatabaseKind::Postgres, Box::new(PostgresDialect)),
        (DatabaseKind::Oracle, Box::new(OracleDialect)),
        (DatabaseKind::SqlServer, Box::new(SqlServerDialect)),
        (DatabaseKind::Sqlite, Box::new(SqliteDialect)),
    ]
}

fn read_supported_datasources() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples").join("supported_datasources.json");
    serde_json::from_str(&fs::read_to_string(path).expect("failed to read supported_datasources.json"))
        .expect("supported_datasources.json must be valid json")
}
