use ora_sql_builder::engine::{
    BuiltQuery, DeleteBuilder, InsertBuilder, JoinType, MetaSqlEngine, Pagination, Predicate,
    Relation, SelectBuilder, SqlServerDialect, TableRef, UpdateBuilder,
};
use serde_json::json;

fn print_query(label: &str, query: BuiltQuery) {
    println!("{} SQL:\n{}", label, query.sql);
    println!("{} Params: {:?}\n", label, query.params);
}

fn main() {
    let engine = MetaSqlEngine::default();
    let dialect = SqlServerDialect;

    let select = engine.build_select(
        &dialect,
        SelectBuilder::new(TableRef::new("meta_relation").alias("mr"))
            .select("mr.id")
            .select("mr.parent_table_id")
            .select_as("child.table_name", "child_table")
            .relation(Relation::new(
                JoinType::Left,
                "mr",
                "child_table_id",
                TableRef::new("meta_table").alias("child"),
                "id",
            ))
            .predicate(Predicate::is_null("mr.deleted_at"))
            .predicate(Predicate::ne("mr.relation_type", "VIRTUAL"))
            .predicate(Predicate::gt("mr.sort_no", 0))
            .predicate(Predicate::gte("mr.created_at", "2026-01-01"))
            .predicate(Predicate::lt("mr.created_at", "2027-01-01"))
            .predicate(Predicate::lte("mr.created_at", "2026-12-31"))
            .predicate(Predicate::like("child.table_name", "%retail%"))
            .predicate(Predicate::in_list("mr.status", vec![json!("ENABLED"), json!("DRAFT")]))
            .predicate(Predicate::between("mr.sort_no", 1, 50))
            .predicate(Predicate::exists(
                "SELECT 1 FROM meta_table parent WHERE parent.id = mr.parent_table_id AND parent.enabled = ?",
                vec![json!(true)],
            ))
            .order_by("mr.id DESC")
            .paginate(Pagination {
                offset: 0,
                limit: 15,
            }),
    );

    let insert = engine.build_insert(
        &dialect,
        InsertBuilder::new("meta_relation")
            .value("id", 4001)
            .value("parent_table_id", 101)
            .value("child_table_id", 202)
            .raw_value("created_at", "GETDATE()"),
    );

    let update = engine.build_update(
        &dialect,
        UpdateBuilder::new("meta_relation")
            .set("relation_name", "门店关联")
            .set("enabled", json!(true))
            .set_raw("updated_at", "GETDATE()")
            .predicate(Predicate::eq("id", 4001)),
    );

    let delete = engine.build_delete(
        &dialect,
        DeleteBuilder::new("meta_relation").predicate(Predicate::eq("id", 4001)),
    );

    print_query("SQL Server SELECT", select);
    print_query("SQL Server INSERT", insert);
    print_query("SQL Server UPDATE", update);
    print_query("SQL Server DELETE", delete);
}
