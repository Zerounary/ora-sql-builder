use ora_sql_builder::engine::{BuiltQuery, PostgresDialect};
use ora_sql_builder::metadata::{
    FieldSource, LinkReference, LinkStep, MetadataField, MetadataQueryOptions, MetadataQueryRequest,
    SortDirection,
};
use ora_sql_builder::metadata_driver::MetadataSqlDriver;
use ora_sql_builder::sql::StatementType;
use serde_json::json;

struct PermissionScope {
    mask_index: usize,
    table_filter: String,
}

struct ImportRow<'a> {
    code: &'a str,
    bill_name: &'a str,
    store_name: &'a str,
    amount: f64,
    enabled: bool,
}

fn print_query(label: &str, query: BuiltQuery) {
    println!("{} SQL:\n{}", label, query.sql);
    println!("{} Params: {:?}\n", label, query.params);
}

fn export_fields() -> Vec<MetadataField> {
    vec![
        MetadataField::new("m_retail", FieldSource::Column("id".to_string())),
        MetadataField::new("m_retail", FieldSource::Column("code".to_string()))
            .with_access("11")
            .with_output_alias("code"),
        MetadataField::new("m_retail", FieldSource::Column("name".to_string()))
            .with_access("11")
            .with_sort(SortDirection::Asc)
            .with_output_alias("bill_name"),
        MetadataField::new("m_retail", FieldSource::Column("amt".to_string()))
            .with_access("01")
            .with_output_alias("sensitive_amount"),
        MetadataField::new("m_retail", FieldSource::Column("store_id".to_string()))
            .with_access("11")
            .with_lookup("c_store", "name")
            .with_output_alias("store_id"),
        MetadataField::new(
            "m_retail",
            FieldSource::Linked(LinkReference {
                steps: vec![
                    LinkStep {
                        foreign_key: "store_id".to_string(),
                        table: "c_store".to_string(),
                    },
                    LinkStep {
                        foreign_key: "org_id".to_string(),
                        table: "c_org".to_string(),
                    },
                ],
                target_column: "name".to_string(),
            }),
        )
        .with_access("11")
        .with_output_alias("org_name"),
    ]
}

fn build_permissioned_export_request(scope: &PermissionScope) -> MetadataQueryRequest {
    MetadataQueryRequest::new(893, StatementType::SELECT, export_fields()).with_options(
        MetadataQueryOptions {
            mask_index: scope.mask_index,
            table_filter: Some(scope.table_filter.clone()),
            ..Default::default()
        },
    )
}

fn build_import_insert_request(row: &ImportRow<'_>, scope: &PermissionScope) -> MetadataQueryRequest {
    MetadataQueryRequest::new(
        893,
        StatementType::INSERT,
        vec![
            MetadataField::new("m_retail", FieldSource::Column("code".to_string()))
                .with_access("11")
                .with_value(json!(row.code)),
            MetadataField::new("m_retail", FieldSource::Column("name".to_string()))
                .with_access("11")
                .with_value(json!(row.bill_name)),
            MetadataField::new("m_retail", FieldSource::Column("docno".to_string()))
                .with_access("11")
                .with_sequence("RET"),
            MetadataField::new("m_retail", FieldSource::Column("store_id".to_string()))
                .with_access("11")
                .with_lookup("c_store", "name")
                .with_value(json!(row.store_name)),
            MetadataField::new("m_retail", FieldSource::Column("amt".to_string()))
                .with_access("01")
                .with_value(json!(row.amount)),
            MetadataField::new("m_retail", FieldSource::Column("enabled".to_string()))
                .with_access("11")
                .with_value(json!(row.enabled)),
        ],
    )
    .with_options(MetadataQueryOptions {
        id: Some(10001),
        mask_index: scope.mask_index,
        table_filter: Some(scope.table_filter.clone()),
        ..Default::default()
    })
}

fn build_import_update_request(
    id: i64,
    row: &ImportRow<'_>,
    scope: &PermissionScope,
) -> MetadataQueryRequest {
    MetadataQueryRequest::new(
        893,
        StatementType::UPDATE,
        vec![
            MetadataField::new("m_retail", FieldSource::Column("name".to_string()))
                .with_access("11")
                .with_value(json!(row.bill_name)),
            MetadataField::new("m_retail", FieldSource::Column("store_id".to_string()))
                .with_access("11")
                .with_lookup("c_store", "name")
                .with_value(json!(row.store_name)),
            MetadataField::new("m_retail", FieldSource::Column("amt".to_string()))
                .with_access("01")
                .with_value(json!(row.amount)),
            MetadataField::new("m_retail", FieldSource::Column("enabled".to_string()))
                .with_access("11")
                .with_value(json!(row.enabled)),
        ],
    )
    .with_options(MetadataQueryOptions {
        id: Some(id),
        mask_index: scope.mask_index,
        table_filter: Some(scope.table_filter.clone()),
        ..Default::default()
    })
}

fn main() {
    let dialect = PostgresDialect;

    let operator_scope = PermissionScope {
        mask_index: 0,
        table_filter: "tenant_id = 37 AND org_id IN (27, 28) AND ownerid = 893".to_string(),
    };
    let auditor_scope = PermissionScope {
        mask_index: 1,
        table_filter: "tenant_id = 37 AND org_id IN (27, 28)".to_string(),
    };

    let import_row = ImportRow {
        code: "RET-10001",
        bill_name: "来自 Excel 的零售单",
        store_name: "一号店",
        amount: 256.8,
        enabled: true,
    };

    print_query(
        "Permissioned LIST for Operator",
        MetadataSqlDriver::new(build_permissioned_export_request(&operator_scope)).build(&dialect),
    );
    print_query(
        "Export SELECT for Auditor",
        MetadataSqlDriver::new(build_permissioned_export_request(&auditor_scope)).build(&dialect),
    );
    print_query(
        "Import INSERT from Metadata Profile",
        MetadataSqlDriver::new(build_import_insert_request(&import_row, &auditor_scope)).build(&dialect),
    );
    print_query(
        "Import UPDATE from Metadata Profile",
        MetadataSqlDriver::new(build_import_update_request(10001, &import_row, &auditor_scope))
            .build(&dialect),
    );
}
