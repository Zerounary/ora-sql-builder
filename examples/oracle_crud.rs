use ora_sql_builder::engine::{
    BuiltQuery, DeleteBuilder, InsertBuilder, JoinType, MetaSqlEngine, OracleDialect, Pagination,
    Predicate, Relation, SelectBuilder, TableRef, UpdateBuilder,
};
use serde_json::json;

fn print_query(label: &str, query: BuiltQuery) {
    println!("{} SQL:\n{}", label, query.sql);
    println!("{} Params: {:?}\n", label, query.params);
}

fn main() {
    let engine = MetaSqlEngine::default();
    let dialect = OracleDialect;

    let select = engine.build_select(
        &dialect,
        SelectBuilder::new(TableRef::new("m_retail").alias("mr"))
            .select("mr.id")
            .select("mr.code")
            .select_as("customer.name", "customer_name")
            .relation(Relation::new(
                JoinType::Inner,
                "mr",
                "customer_id",
                TableRef::new("c_customer").alias("customer"),
                "id",
            ))
            .predicate(Predicate::eq("mr.owner_id", 893))
            .predicate(Predicate::ne("mr.status", "VOID"))
            .predicate(Predicate::gt("mr.qty", 0))
            .predicate(Predicate::gte("mr.bill_date", "2026-01-01"))
            .predicate(Predicate::lt("mr.bill_date", "2026-02-01"))
            .predicate(Predicate::lte("mr.bill_date", "2026-01-31"))
            .predicate(Predicate::like("customer.name", "%商贸%"))
            .predicate(Predicate::in_list("mr.status", vec![json!("OPEN"), json!("DONE")]))
            .predicate(Predicate::between("mr.amt", 10, 200))
            .predicate(Predicate::exists(
                "SELECT 1 FROM m_retail_line line WHERE line.bill_id = mr.id AND line.enabled = ?",
                vec![json!("Y")],
            ))
            .order_by("mr.id DESC")
            .paginate(Pagination {
                offset: 10,
                limit: 10,
            }),
    );

    let insert = engine.build_insert(
        &dialect,
        InsertBuilder::new("m_retail")
            .value("id", 3001)
            .value("code", "OR-3001")
            .value("amt", json!(99.5))
            .raw_value("created_at", "sysdate")
            .raw_value("docno", "get_sequenceno('OR', 37)"),
    );

    let update = engine.build_update(
        &dialect,
        UpdateBuilder::new("m_retail")
            .set("amt", json!(199.5))
            .set("status", "APPROVED")
            .set_raw("modifieddate", "sysdate")
            .predicate(Predicate::eq("id", 3001)),
    );

    let delete = engine.build_delete(
        &dialect,
        DeleteBuilder::new("m_retail").predicate(Predicate::eq("id", 3001)),
    );

    print_query("Oracle SELECT", select);
    print_query("Oracle INSERT", insert);
    print_query("Oracle UPDATE", update);
    print_query("Oracle DELETE", delete);
}
