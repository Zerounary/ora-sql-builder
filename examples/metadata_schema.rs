use ora_sql_builder::engine::{BuiltQuery, MetaSqlEngine, PostgresDialect};
use ora_sql_builder::metadata::standard_metadata_tables;

fn print_query(label: &str, query: BuiltQuery) {
    println!("{} SQL:\n{}", label, query.sql);
    println!("{} Params: {:?}\n", label, query.params);
}

fn main() {
    let engine = MetaSqlEngine::default();
    let dialect = PostgresDialect;

    for table in standard_metadata_tables() {
        let label = format!("Create {}", table.name);
        let query = engine.build_create_table(&dialect, table.to_create_table_builder().if_not_exists());
        print_query(&label, query);
    }
}
