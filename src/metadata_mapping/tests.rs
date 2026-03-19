use similar_asserts::assert_eq;

use super::*;
use crate::metadata::{
    DatabaseKind, MetaColumn, MetaDatasource, MetaExportProfile, MetaImportFieldMapping,
    MetaImportProfile, MetaPolicy, MetaRelation, MetaTable, MetadataCatalog,
    MetadataColumnType, MetadataFilterExpr, PolicyKind, RelationKind,
    standard_metadata_tables,
};

#[test]
fn snapshot_covers_standard_entities_and_extended_profile_tables() {
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
        .relation(MetaRelation::new(200, 10, 11, RelationKind::ManyToOne, "store_id", "id"))
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

    assert_eq!(snapshot.rows_for("meta_datasource").len(), 1);
    assert_eq!(snapshot.rows_for("meta_table").len(), 1);
    assert_eq!(snapshot.rows_for("meta_column").len(), 1);
    assert_eq!(snapshot.rows_for("meta_relation").len(), 1);
    assert_eq!(snapshot.rows_for("meta_policy").len(), 1);
    assert_eq!(snapshot.rows_for("meta_import_profile").len(), 1);
    assert_eq!(snapshot.rows_for("meta_import_mapping").len(), 1);
    assert_eq!(snapshot.rows_for("meta_export_profile").len(), 1);
    assert!(snapshot.schemas.iter().any(|table| table.name == "meta_export_profile"));
    assert!(snapshot.schemas.iter().any(|table| table.name == "meta_import_mapping"));
}

#[test]
fn snapshot_can_produce_ddl_builders_from_same_schema_source() {
    let snapshot = MetadataPersistenceSnapshot {
        schemas: standard_metadata_tables(),
        rows: Vec::new(),
    };

    let builders = snapshot.ddl_builders();

    assert_eq!(builders.len(), standard_metadata_tables().len());
}
