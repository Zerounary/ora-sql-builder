use ora_sql_builder::engine::{
    BuiltQuery, DeleteBuilder, InsertBuilder, MetaSqlEngine, Pagination, Predicate, SelectBuilder,
    SqliteDialect, TableRef, UpdateBuilder,
};
use serde_json::json;

fn print_query(label: &str, query: BuiltQuery) {
    println!("{} SQL:\n{}", label, query.sql);
    println!("{} Params: {:?}\n", label, query.params);
}

fn main() {
    let engine = MetaSqlEngine::default();
    let dialect = SqliteDialect;

    let select = engine.build_select(
        &dialect,
        SelectBuilder::new(TableRef::new("meta_table").alias("mt"))
            .select("mt.id")
            .select("mt.table_name")
            .predicate(Predicate::eq("mt.owner_id", 893))
            .predicate(Predicate::ne("mt.table_type", "VIEW"))
            .predicate(Predicate::gt("mt.sort_no", 0))
            .predicate(Predicate::gte("mt.created_at", "2026-01-01"))
            .predicate(Predicate::lt("mt.created_at", "2027-01-01"))
            .predicate(Predicate::lte("mt.created_at", "2026-12-31"))
            .predicate(Predicate::like("mt.table_name", "%retail%"))
            .predicate(Predicate::in_list("mt.status", vec![json!("OPEN"), json!("DONE")]))
            .predicate(Predicate::between("mt.sort_no", 1, 99))
            .predicate(Predicate::exists(
                "SELECT 1 FROM meta_column mc WHERE mc.table_id = mt.id AND mc.enabled = ?",
                vec![json!(true)],
            ))
            .order_by("mt.id DESC")
            .paginate(Pagination {
                offset: 0,
                limit: 20,
            }),
    );

    let insert = engine.build_insert(
        &dialect,
        InsertBuilder::new("meta_table")
            .value("id", 5001)
            .value("table_name", "m_retail")
            .value("display_name", "SQLite 零售单据")
            .value("enabled", true)
            .raw_value("created_at", "CURRENT_TIMESTAMP"),
    );

    let update = engine.build_update(
        &dialect,
        UpdateBuilder::new("meta_table")
            .set("display_name", "SQLite 零售单据-已更新")
            .set("enabled", false)
            .set_raw("updated_at", "CURRENT_TIMESTAMP")
            .predicate(Predicate::eq("id", 5001)),
    );

    let delete = engine.build_delete(
        &dialect,
        DeleteBuilder::new("meta_table")
            .predicate(Predicate::raw("tenant_id = 37"))
            .predicate(Predicate::eq("id", 5001)),
    );

    print_query("SQLite SELECT", select);
    print_query("SQLite INSERT", insert);
    print_query("SQLite UPDATE", update);
    print_query("SQLite DELETE", delete);
}
