use std::hint::black_box;
use std::mem::size_of;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::json;
use serde_json::Value;
use similar_asserts::assert_eq;

use super::*;

#[test]
fn build_postgres_select_with_relation_predicates_and_pagination() {
    let engine = MetaSqlEngine;
    let dialect = PostgresDialect;
    let query = engine.build_select(
        &dialect,
        SelectBuilder::new(TableRef::new("m_retail").alias("mr"))
            .select("mr.id")
            .select_as("store.name", "store_name")
            .relation(Relation::new(
                JoinType::Left,
                "mr",
                "store_id",
                TableRef::new("c_store").alias("store"),
                "id",
            ))
            .predicate(Predicate::eq("mr.owner_id", 893))
            .predicate(Predicate::like("store.name", "%旗舰%"))
            .predicate(Predicate::in_list("mr.status", vec![json!("OPEN"), json!("CLOSED")]))
            .group_by("mr.id")
            .group_by("store.name")
            .order_by("mr.id DESC")
            .paginate(Pagination {
                offset: 20,
                limit: 10,
            }),
    );

    assert_eq!(
        query.sql,
        "SELECT mr.id, store.name AS store_name FROM m_retail mr LEFT JOIN c_store store ON mr.store_id = store.id WHERE mr.owner_id = $1 AND store.name LIKE $2 AND mr.status IN ($3, $4) GROUP BY mr.id, store.name ORDER BY mr.id DESC LIMIT 10 OFFSET 20".to_string()
    );
    assert_eq!(query.params, vec![json!(893), json!("%旗舰%"), json!("OPEN"), json!("CLOSED")]);
}

#[test]
fn build_mysql_insert_uses_question_mark_placeholders() {
    let engine = MetaSqlEngine;
    let dialect = MySqlDialect;
    let query = engine.build_insert(
        &dialect,
        InsertBuilder::new("m_retail")
            .value("id", 1)
            .value("code", "RE-001")
            .value("enabled", true),
    );

    assert_eq!(
        query.sql,
        "INSERT INTO m_retail (id, code, enabled) VALUES (?, ?, ?)".to_string()
    );
    assert_eq!(query.params, vec![json!(1), json!("RE-001"), json!(true)]);
}

#[test]
fn build_oracle_update_uses_numbered_placeholders_in_order() {
    let engine = MetaSqlEngine;
    let dialect = OracleDialect;
    let query = engine.build_update(
        &dialect,
        UpdateBuilder::new("m_retail")
            .set("name", "新名称")
            .set("qty", 30)
            .predicate(Predicate::gte("modified_date", "2026-01-01"))
            .predicate(Predicate::eq("id", 1)),
    );

    assert_eq!(
        query.sql,
        "UPDATE m_retail SET name = :1, qty = :2 WHERE modified_date >= :3 AND id = :4".to_string()
    );
    assert_eq!(
        query.params,
        vec![json!("新名称"), json!(30), json!("2026-01-01"), json!(1)]
    );
}

#[test]
fn build_sql_server_delete_adds_fallback_order_by_for_pagination() {
    let dialect = SqlServerDialect;
    let query = SelectBuilder::new(TableRef::new("meta_table"))
        .select("id")
        .predicate(Predicate::is_null("deleted_at"))
        .paginate(Pagination {
            offset: 5,
            limit: 15,
        })
        .build(&dialect);

    assert_eq!(
        query.sql,
        "SELECT id FROM meta_table WHERE deleted_at IS NULL ORDER BY (SELECT 1) OFFSET 5 ROWS FETCH NEXT 15 ROWS ONLY".to_string()
    );
    assert_eq!(query.params, Vec::<Value>::new());
}

#[test]
fn build_delete_with_raw_and_empty_in_list_is_safe() {
    let engine = MetaSqlEngine;
    let dialect = PostgresDialect;
    let query = engine.build_delete(
        &dialect,
        DeleteBuilder::new("meta_operation_log")
            .predicate(Predicate::raw("tenant_id = 37"))
            .predicate(Predicate::in_list("id", Vec::new())),
    );

    assert_eq!(
        query.sql,
        "DELETE FROM meta_operation_log WHERE tenant_id = 37 AND 1 = 0".to_string()
    );
    assert_eq!(query.params, Vec::<Value>::new());
}

#[test]
fn build_sql_server_select_keeps_explicit_order_by_when_paginating() {
    let dialect = SqlServerDialect;
    let query = SelectBuilder::new(TableRef::new("meta_table"))
        .order_by("id DESC")
        .paginate(Pagination {
            offset: 0,
            limit: 5,
        })
        .build(&dialect);

    assert_eq!(
        query.sql,
        "SELECT * FROM meta_table ORDER BY id DESC OFFSET 0 ROWS FETCH NEXT 5 ROWS ONLY"
            .to_string()
    );
    assert_eq!(query.params, Vec::<Value>::new());
}

#[test]
fn build_oracle_select_wraps_paginated_query() {
    let dialect = OracleDialect;
    let query = SelectBuilder::new(TableRef::new("meta_table"))
        .select("id")
        .paginate(Pagination {
            offset: 10,
            limit: 20,
        })
        .build(&dialect);

    assert_eq!(
        query.sql,
        "SELECT * FROM (SELECT inner_query.*, ROWNUM AS row_num FROM (SELECT id FROM meta_table) inner_query WHERE ROWNUM <= 30) WHERE row_num > 10"
            .to_string()
    );
    assert_eq!(query.params, Vec::<Value>::new());
}

#[test]
fn build_inner_join_select_renders_join_keyword() {
    let dialect = PostgresDialect;
    let query = SelectBuilder::new(TableRef::new("m_order").alias("o"))
        .select("o.id")
        .relation(Relation::new(
            JoinType::Inner,
            "o",
            "customer_id",
            TableRef::new("c_customer").alias("c"),
            "id",
        ))
        .build(&dialect);

    assert_eq!(
        query.sql,
        "SELECT o.id FROM m_order o INNER JOIN c_customer c ON o.customer_id = c.id"
            .to_string()
    );
    assert_eq!(query.params, Vec::<Value>::new());
}

#[test]
fn build_select_supports_advanced_predicates() {
    let dialect = PostgresDialect;
    let query = SelectBuilder::new(TableRef::new("m_retail").alias("mr"))
        .select("mr.id")
        .predicate(Predicate::eq("mr.owner_id", 893))
        .predicate(Predicate::ne("mr.status", "CANCELLED"))
        .predicate(Predicate::gt("mr.qty", 0))
        .predicate(Predicate::gte("mr.bill_date", "2026-01-01"))
        .predicate(Predicate::lt("mr.bill_date", "2026-02-01"))
        .predicate(Predicate::lte("mr.bill_date", "2026-01-31"))
        .predicate(Predicate::between("mr.amt", 10, 99))
        .predicate(Predicate::exists(
            "SELECT 1 FROM c_store s WHERE s.id = mr.store_id AND s.enabled = ?",
            vec![json!("Y")],
        ))
        .build(&dialect);

    assert_eq!(
        query.sql,
        "SELECT mr.id FROM m_retail mr WHERE mr.owner_id = $1 AND mr.status != $2 AND mr.qty > $3 AND mr.bill_date >= $4 AND mr.bill_date < $5 AND mr.bill_date <= $6 AND mr.amt BETWEEN $7 AND $8 AND EXISTS (SELECT 1 FROM c_store s WHERE s.id = mr.store_id AND s.enabled = $9)"
            .to_string()
    );
    assert_eq!(
        query.params,
        vec![
            json!(893),
            json!("CANCELLED"),
            json!(0),
            json!("2026-01-01"),
            json!("2026-02-01"),
            json!("2026-01-31"),
            json!(10),
            json!(99),
            json!("Y"),
        ]
    );
}

#[test]
fn build_insert_and_update_support_raw_sql_expressions() {
    let dialect = OracleDialect;
    let insert = InsertBuilder::new("m_retail")
        .value("id", 1)
        .raw_value("created_at", "sysdate")
        .raw_value("docno", "get_sequenceno('RE', 37)")
        .build(&dialect);
    let update = UpdateBuilder::new("m_retail")
        .set("modifierid", 893)
        .set_raw("modifieddate", "sysdate")
        .predicate(Predicate::eq("id", 1))
        .build(&dialect);

    assert_eq!(
        insert.sql,
        "INSERT INTO m_retail (id, created_at, docno) VALUES (:1, sysdate, get_sequenceno('RE', 37))"
            .to_string()
    );
    assert_eq!(insert.params, vec![json!(1)]);

    assert_eq!(
        update.sql,
        "UPDATE m_retail SET modifierid = :1, modifieddate = sysdate WHERE id = :2"
            .to_string()
    );
    assert_eq!(update.params, vec![json!(893), json!(1)]);
}

#[test]
fn build_sqlite_select_uses_question_mark_placeholders_and_limit_offset() {
    let dialect = SqliteDialect;
    let query = SelectBuilder::new(TableRef::new("meta_table"))
        .select("id")
        .predicate(Predicate::eq("owner_id", 37))
        .predicate(Predicate::between("sort_no", 1, 10))
        .paginate(Pagination {
            offset: 5,
            limit: 10,
        })
        .build(&dialect);

    assert_eq!(
        query.sql,
        "SELECT id FROM meta_table WHERE owner_id = ? AND sort_no BETWEEN ? AND ? LIMIT 10 OFFSET 5"
            .to_string()
    );
    assert_eq!(query.params, vec![json!(37), json!(1), json!(10)]);
}

#[test]
fn build_create_table_supports_metadata_driven_schema_definition() {
    let engine = MetaSqlEngine;
    let dialect = PostgresDialect;
    let query = engine.build_create_table(
        &dialect,
        CreateTableBuilder::new("meta_export_profile")
            .if_not_exists()
            .column(ColumnDefinition::new("id", "BIGINT").not_null())
            .column(ColumnDefinition::new("profile_code", "VARCHAR(64)").not_null().unique())
            .column(
                ColumnDefinition::new("enabled", "BOOLEAN")
                    .not_null()
                    .default_value(true),
            )
            .column(ColumnDefinition::new("datasource_id", "BIGINT").not_null())
            .primary_key(vec!["id"])
            .foreign_key(
                ForeignKeyDefinition::new(
                    vec!["datasource_id"],
                    "meta_datasource",
                    vec!["id"],
                )
                .name("fk_meta_export_profile_datasource")
                .on_delete("CASCADE"),
            ),
    );

    assert_eq!(
        query.sql,
        "CREATE TABLE IF NOT EXISTS meta_export_profile (id BIGINT NOT NULL, profile_code VARCHAR(64) NOT NULL UNIQUE, enabled BOOLEAN NOT NULL DEFAULT TRUE, datasource_id BIGINT NOT NULL, PRIMARY KEY (id), CONSTRAINT fk_meta_export_profile_datasource FOREIGN KEY (datasource_id) REFERENCES meta_datasource (id) ON DELETE CASCADE)"
            .to_string()
    );
    assert_eq!(query.params, Vec::<Value>::new());
}

#[test]
fn build_alter_and_drop_table_support_schema_evolution() {
    let engine = MetaSqlEngine;
    let dialect = SqliteDialect;
    let alter = engine.build_alter_table(
        &dialect,
        AlterTableBuilder::new("meta_export_profile")
            .add_column(
                ColumnDefinition::new("tenant_scope", "VARCHAR(128)")
                    .not_null()
                    .default_value("ALL"),
            )
            .rename_column("profile_code", "profile_key")
            .add_constraint("ADD CONSTRAINT uq_meta_export_profile_key UNIQUE (profile_key)"),
    );
    let drop = engine.build_drop_table(
        &dialect,
        DropTableBuilder::new("meta_export_profile_backup").if_exists(),
    );

    assert_eq!(
        alter.sql,
        "ALTER TABLE meta_export_profile ADD COLUMN tenant_scope VARCHAR(128) NOT NULL DEFAULT 'ALL'; ALTER TABLE meta_export_profile RENAME COLUMN profile_code TO profile_key; ALTER TABLE meta_export_profile ADD CONSTRAINT uq_meta_export_profile_key UNIQUE (profile_key)"
            .to_string()
    );
    assert_eq!(alter.params, Vec::<Value>::new());

    assert_eq!(
        drop.sql,
        "DROP TABLE IF EXISTS meta_export_profile_backup".to_string()
    );
    assert_eq!(drop.params, Vec::<Value>::new());
}

#[test]
#[ignore]
fn concurrent_builder_performance_report() {
    let report = vec![
        benchmark_case("select", || {
            black_box(
                SelectBuilder::new(TableRef::new("m_retail").alias("mr"))
                    .select("mr.id")
                    .select("mr.code")
                    .relation(Relation::new(
                        JoinType::Left,
                        "mr",
                        "store_id",
                        TableRef::new("c_store").alias("store"),
                        "id",
                    ))
                    .predicate(Predicate::and(vec![
                        Predicate::eq("mr.owner_id", 893),
                        Predicate::or(vec![
                            Predicate::like("store.name", "%旗舰%"),
                            Predicate::like("store.name", "%门店%"),
                        ]),
                        Predicate::between("mr.amt", 10, 99),
                        Predicate::exists(
                            "SELECT 1 FROM m_retail_line line WHERE line.bill_id = mr.id AND line.enabled = ?",
                            vec![json!("Y")],
                        ),
                    ]))
                    .order_by("mr.id DESC")
                    .build(&PostgresDialect),
            );
        }),
        benchmark_case("insert", || {
            black_box(
                InsertBuilder::new("m_retail")
                    .value("id", 1)
                    .value("code", "RE-001")
                    .value("name", "性能测试")
                    .value("enabled", true)
                    .raw_value("created_at", "CURRENT_TIMESTAMP")
                    .build(&SqliteDialect),
            );
        }),
        benchmark_case("update", || {
            black_box(
                UpdateBuilder::new("m_retail")
                    .set("name", "性能测试-更新")
                    .set("enabled", false)
                    .set_raw("updated_at", "CURRENT_TIMESTAMP")
                    .predicate(Predicate::eq("id", 1))
                    .build(&PostgresDialect),
            );
        }),
        benchmark_case("delete", || {
            black_box(
                DeleteBuilder::new("m_retail")
                    .predicate(Predicate::raw("tenant_id = 37"))
                    .predicate(Predicate::eq("id", 1))
                    .build(&PostgresDialect),
            );
        }),
        benchmark_case("ddl_create", || {
            black_box(
                CreateTableBuilder::new("meta_perf_case")
                    .if_not_exists()
                    .column(ColumnDefinition::new("id", "BIGINT").not_null())
                    .column(ColumnDefinition::new("code", "VARCHAR(64)").not_null().unique())
                    .primary_key(vec!["id"])
                    .build(&PostgresDialect),
            );
        }),
    ];

    println!("engine concurrent performance report");
    println!(
        "struct_sizes => SelectBuilder={}B, InsertBuilder={}B, UpdateBuilder={}B, DeleteBuilder={}B, CreateTableBuilder={}B, Predicate={}B, Relation={}B, TableRef={}B, BuiltQuery={}B",
        size_of::<SelectBuilder>(),
        size_of::<InsertBuilder>(),
        size_of::<UpdateBuilder>(),
        size_of::<DeleteBuilder>(),
        size_of::<CreateTableBuilder>(),
        size_of::<Predicate>(),
        size_of::<Relation>(),
        size_of::<TableRef>(),
        size_of::<BuiltQuery>(),
    );
    for line in report {
        println!("{}", line);
    }
}

fn benchmark_case<F>(name: &str, task: F) -> String
where
    F: Fn() + Send + Sync + 'static,
{
    const THREADS: usize = 8;
    const ITERATIONS_PER_THREAD: usize = 5_000;

    let task = Arc::new(task);
    let start = Instant::now();
    let mut handles = Vec::with_capacity(THREADS);
    for _ in 0..THREADS {
        let task = Arc::clone(&task);
        handles.push(thread::spawn(move || {
            let thread_start = Instant::now();
            for _ in 0..ITERATIONS_PER_THREAD {
                task();
            }
            thread_start.elapsed()
        }));
    }

    let mut longest_thread = Duration::ZERO;
    for handle in handles {
        let elapsed = handle.join().expect("engine performance worker panicked");
        if elapsed > longest_thread {
            longest_thread = elapsed;
        }
    }
    let total_elapsed = start.elapsed();
    let total_ops = THREADS * ITERATIONS_PER_THREAD;
    let avg_ns = total_elapsed.as_nanos() / total_ops as u128;

    format!(
        "case={} total_ops={} total_elapsed_ms={} longest_thread_ms={} avg_ns_per_op={}",
        name,
        total_ops,
        total_elapsed.as_millis(),
        longest_thread.as_millis(),
        avg_ns,
    )
}
