use ora_sql_builder::engine::{
    BuiltQuery, DeleteBuilder, InsertBuilder, MetaSqlEngine, Pagination, PostgresDialect,
    Predicate, SelectBuilder, TableRef, UpdateBuilder,
};

fn print_query(label: &str, query: BuiltQuery) {
    println!("{} SQL:\n{}", label, query.sql);
    println!("{} Params: {:?}\n", label, query.params);
}

fn main() {
    let engine = MetaSqlEngine::default();
    let dialect = PostgresDialect;

    let select = engine.build_select(
        &dialect,
        SelectBuilder::new(TableRef::new("meta_table").alias("mt"))
            .select("mt.id")
            .select("mt.table_name")
            .predicate(Predicate::like("mt.table_name", "%retail%"))
            .predicate(Predicate::gte("mt.created_at", "2026-01-01"))
            .predicate(Predicate::lte("mt.created_at", "2026-12-31"))
            .order_by("mt.id DESC")
            .paginate(Pagination {
                offset: 20,
                limit: 10,
            }),
    );

    let insert = engine.build_insert(
        &dialect,
        InsertBuilder::new("meta_table")
            .value("id", 2001)
            .value("table_name", "m_retail")
            .value("display_name", "零售单据"),
    );

    let update = engine.build_update(
        &dialect,
        UpdateBuilder::new("meta_table")
            .set("display_name", "零售单据-已更新")
            .predicate(Predicate::eq("id", 2001)),
    );

    let delete = engine.build_delete(
        &dialect,
        DeleteBuilder::new("meta_table")
            .predicate(Predicate::raw("tenant_id = 37"))
            .predicate(Predicate::eq("id", 2001)),
    );

    print_query("PostgreSQL SELECT", select);
    print_query("PostgreSQL INSERT", insert);
    print_query("PostgreSQL UPDATE", update);
    print_query("PostgreSQL DELETE", delete);
}
