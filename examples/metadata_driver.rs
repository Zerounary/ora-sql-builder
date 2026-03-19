use ora_sql_builder::engine::{BuiltQuery, PostgresDialect};
use ora_sql_builder::metadata::{
    FieldSource, LinkReference, LinkStep, MetadataField, MetadataFilterExpr,
    MetadataQueryOptions, MetadataQueryRequest, SortDirection,
};
use ora_sql_builder::metadata_driver::MetadataSqlDriver;
use ora_sql_builder::sql::StatementType;
use serde_json::json;

fn print_query(label: &str, query: BuiltQuery) {
    println!("{} SQL:\n{}", label, query.sql);
    println!("{} Params: {:?}\n", label, query.params);
}

fn main() {
    let dialect = PostgresDialect;

    let select_request = MetadataQueryRequest::new(
        893,
        StatementType::SELECT,
        vec![
            MetadataField::new("m_retail", FieldSource::Column("id".to_string())),
            MetadataField::new("m_retail", FieldSource::Column("name".to_string()))
                .with_access("1")
                .with_sort(SortDirection::Asc)
                .with_output_alias("name"),
            MetadataField::new("m_retail", FieldSource::Column("enabled".to_string()))
                .with_access("1")
                .with_output_alias("enabled"),
            MetadataField::new("m_retail", FieldSource::Column("amt".to_string()))
                .with_access("1")
                .with_output_alias("amt"),
            MetadataField::new("m_retail", FieldSource::Column("store_id".to_string()))
                .with_access("1")
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
                            foreign_key: "store_kind_id".to_string(),
                            table: "c_store_kind".to_string(),
                        },
                    ],
                    target_column: "name".to_string(),
                }),
            )
            .with_access("1")
            .with_output_alias("store_kind_name"),
        ],
    )
    .with_options(MetadataQueryOptions {
        table_filter: Some("tenant_id = 37".to_string()),
        ..Default::default()
    })
    .with_filters(vec![MetadataFilterExpr::and(vec![
        MetadataFilterExpr::or(vec![
            MetadataFilterExpr::like("name", "%旗舰%"),
            MetadataFilterExpr::like("name", "%门店%"),
        ]),
        MetadataFilterExpr::eq("enabled", true),
        MetadataFilterExpr::between("amt", 10, 99),
        MetadataFilterExpr::exists(
            "SELECT 1 FROM c_store s WHERE s.id = m_retail.store_id AND s.enabled = ?",
            vec![json!("Y")],
        ),
    ])]);

    let grouped_request = MetadataQueryRequest::new(
        893,
        StatementType::SELECT,
        vec![
            MetadataField::new("m_retail", FieldSource::Column("id".to_string())),
            MetadataField::new("m_retail", FieldSource::Column("dept_name".to_string()))
                .with_access("1")
                .with_output_alias("dept_name"),
            MetadataField::new("m_retail", FieldSource::Formula("sum(qty)".to_string()))
                .with_access("1")
                .with_output_alias("total_qty"),
        ],
    )
    .with_options(MetadataQueryOptions {
        grouped: true,
        ..Default::default()
    })
    .with_having(vec![MetadataFilterExpr::gt("total_qty", 100)]);

    let insert_request = MetadataQueryRequest::new(
        893,
        StatementType::INSERT,
        vec![
            MetadataField::new("m_retail", FieldSource::Column("code".to_string()))
                .with_access("1")
                .with_default(json!("默认值")),
            MetadataField::new("m_retail", FieldSource::Column("name".to_string()))
                .with_access("1")
                .with_value(json!("名称")),
            MetadataField::new("m_retail", FieldSource::Column("docno".to_string()))
                .with_access("1")
                .with_sequence("RE"),
            MetadataField::new("m_retail", FieldSource::Column("customer_id".to_string()))
                .with_access("1")
                .with_lookup("c_store", "name")
                .with_value(json!("一号店")),
        ],
    )
    .with_options(MetadataQueryOptions {
        id: Some(1),
        ..Default::default()
    });

    let update_request = MetadataQueryRequest::new(
        893,
        StatementType::UPDATE,
        vec![
            MetadataField::new("m_retail", FieldSource::Column("name".to_string()))
                .with_access("1")
                .with_value(json!("已更新名称")),
            MetadataField::new("m_retail", FieldSource::Column("enabled".to_string()))
                .with_access("1")
                .with_value(json!(true)),
        ],
    )
    .with_options(MetadataQueryOptions {
        id: Some(1),
        table_filter: Some("tenant_id = 37".to_string()),
        ..Default::default()
    });

    let delete_request = MetadataQueryRequest::new(
        893,
        StatementType::DELETE,
        vec![MetadataField::new(
            "m_retail",
            FieldSource::Column("id".to_string()),
        )],
    )
    .with_options(MetadataQueryOptions {
        id: Some(1),
        table_filter: Some("tenant_id = 37".to_string()),
        ..Default::default()
    });

    print_query(
        "Metadata SELECT",
        MetadataSqlDriver::new(select_request).build(&dialect),
    );
    print_query(
        "Metadata GROUPED SELECT",
        MetadataSqlDriver::new(grouped_request).build(&dialect),
    );
    print_query(
        "Metadata INSERT",
        MetadataSqlDriver::new(insert_request).build(&dialect),
    );
    print_query(
        "Metadata UPDATE",
        MetadataSqlDriver::new(update_request).build(&dialect),
    );
    print_query(
        "Metadata DELETE",
        MetadataSqlDriver::new(delete_request).build(&dialect),
    );
}
