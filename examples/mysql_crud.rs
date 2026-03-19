use ora_sql_builder::engine::{
    BuiltQuery, DeleteBuilder, InsertBuilder, JoinType, MetaSqlEngine, MySqlDialect, Pagination,
    Predicate, Relation, SelectBuilder, TableRef, UpdateBuilder,
};
use serde_json::json;

fn print_query(label: &str, query: BuiltQuery) {
    println!("{} SQL:\n{}", label, query.sql);
    println!("{} Params: {:?}\n", label, query.params);
}

fn main() {
    let engine = MetaSqlEngine::default();
    let dialect = MySqlDialect;

    let select = engine.build_select(
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
            .predicate(Predicate::like("store.name", "%旗舰店%"))
            .predicate(Predicate::in_list("mr.status", vec![json!("OPEN"), json!("CLOSED")]))
            .order_by("mr.id DESC")
            .paginate(Pagination {
                offset: 0,
                limit: 20,
            }),
    );

    let insert = engine.build_insert(
        &dialect,
        InsertBuilder::new("m_retail")
            .value("id", 1001)
            .value("code", "RE-1001")
            .value("name", "MySQL 零售单")
            .value("enabled", true),
    );

    let update = engine.build_update(
        &dialect,
        UpdateBuilder::new("m_retail")
            .set("name", "MySQL 零售单-已更新")
            .set("status", "DONE")
            .predicate(Predicate::eq("id", 1001)),
    );

    let delete = engine.build_delete(
        &dialect,
        DeleteBuilder::new("m_retail").predicate(Predicate::eq("id", 1001)),
    );

    print_query("MySQL SELECT", select);
    print_query("MySQL INSERT", insert);
    print_query("MySQL UPDATE", update);
    print_query("MySQL DELETE", delete);
}
