use ora_sql_builder::engine::{MetaSqlEngine, PostgresDialect};
use ora_sql_builder::metadata::{
    DatabaseKind, MetaColumn, MetaDatasource, MetaExportProfile, MetaImportFieldMapping,
    MetaImportProfile, MetaPolicy, MetaTable, MetadataCatalog, MetadataColumnType,
    MetadataFilterExpr, PolicyKind,
};
use ora_sql_builder::metadata_mapping::MetadataPersistenceMapper;

fn main() {
    let catalog = MetadataCatalog::new()
        .datasource(MetaDatasource::new(
            1,
            "main",
            "主数据源",
            DatabaseKind::Postgres,
            "postgres://demo",
        ))
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

    let snapshot = MetadataPersistenceMapper::snapshot_from_catalog(&catalog);
    let engine = MetaSqlEngine::default();
    let dialect = PostgresDialect;

    println!("Snapshot rows:\n{:#?}\n", snapshot.rows);
    for schema in snapshot.schemas.iter() {
        let query = engine.build_create_table(&dialect, schema.to_create_table_builder().if_not_exists());
        println!("DDL for {}:\n{}\n", schema.name, query.sql);
    }
}
