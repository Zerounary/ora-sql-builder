use serde_json::json;

use crate::metadata::{
    DatabaseKind, FieldSource, LinkReference, LinkStep, MetaDatasource, MetadataColumnSchema,
    MetadataColumnType, MetadataField, MetadataFilterExpr, MetadataQueryOptions,
    MetadataQueryRequest, MetadataTableSchema, SortDirection,
};
use crate::sql::StatementType;

pub fn sqlite_demo_datasource() -> MetaDatasource {
    MetaDatasource::new(
        1,
        "sqlite_demo",
        "SQLite Demo Datasource",
        DatabaseKind::Sqlite,
        "sqlite::memory:",
    )
    .with_options(json!({"max_connections": 1}))
}

pub fn demo_sales_table_schemas() -> Vec<MetadataTableSchema> {
    vec![
        MetadataTableSchema::new("demo_store")
            .column(MetadataColumnSchema::new("id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("name", MetadataColumnType::Varchar(64)).not_null())
            .column(MetadataColumnSchema::new("region", MetadataColumnType::Varchar(64)).not_null())
            .primary_key(vec!["id"]),
        MetadataTableSchema::new("demo_customer")
            .column(MetadataColumnSchema::new("id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("name", MetadataColumnType::Varchar(64)).not_null())
            .column(MetadataColumnSchema::new("level", MetadataColumnType::Varchar(32)).not_null())
            .column(MetadataColumnSchema::new("enabled", MetadataColumnType::Varchar(8)).not_null())
            .primary_key(vec!["id"]),
        MetadataTableSchema::new("demo_sale_order")
            .column(MetadataColumnSchema::new("id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("ad_client_id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("ad_org_id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("ownerid", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("modifiered", MetadataColumnType::BigInt).not_null())
            .column(
                MetadataColumnSchema::new("modifierid", MetadataColumnType::BigInt)
                    .not_null()
                    .default_raw("0"),
            )
            .column(MetadataColumnSchema::new("creationdate", MetadataColumnType::DateTime).not_null())
            .column(MetadataColumnSchema::new("modifieddate", MetadataColumnType::DateTime).not_null())
            .column(MetadataColumnSchema::new("code", MetadataColumnType::Varchar(64)).not_null())
            .column(MetadataColumnSchema::new("store_id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("customer_id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("status", MetadataColumnType::Varchar(32)).not_null())
            .column(MetadataColumnSchema::new("total_amount", MetadataColumnType::Decimal { precision: 18, scale: 2 }).not_null())
            .column(MetadataColumnSchema::new("tenant_id", MetadataColumnType::BigInt).not_null())
            .column(MetadataColumnSchema::new("enabled", MetadataColumnType::Varchar(8)).not_null())
            .primary_key(vec!["id"]),
    ]
}

pub fn order_query_request() -> MetadataQueryRequest {
    MetadataQueryRequest::new(
        893,
        StatementType::SELECT,
        vec![
            MetadataField::new("demo_sale_order", FieldSource::Column("id".to_string())),
            MetadataField::new("demo_sale_order", FieldSource::Column("code".to_string()))
                .with_access("1")
                .with_sort(SortDirection::Asc)
                .with_output_alias("order_code"),
            MetadataField::new("demo_sale_order", FieldSource::Column("status".to_string()))
                .with_access("1")
                .with_output_alias("status"),
            MetadataField::new("demo_sale_order", FieldSource::Column("total_amount".to_string()))
                .with_access("1")
                .with_output_alias("total_amount"),
            MetadataField::new(
                "demo_sale_order",
                FieldSource::Linked(LinkReference {
                    steps: vec![LinkStep {
                        foreign_key: "store_id".to_string(),
                        table: "demo_store".to_string(),
                    }],
                    target_column: "name".to_string(),
                }),
            )
            .with_access("1")
            .with_output_alias("store_name"),
            MetadataField::new(
                "demo_sale_order",
                FieldSource::Linked(LinkReference {
                    steps: vec![LinkStep {
                        foreign_key: "customer_id".to_string(),
                        table: "demo_customer".to_string(),
                    }],
                    target_column: "name".to_string(),
                }),
            )
            .with_access("1")
            .with_output_alias("customer_name"),
        ],
    )
    .with_options(MetadataQueryOptions {
        table_filter: Some("tenant_id = 37".to_string()),
        ..Default::default()
    })
    .with_filters(vec![MetadataFilterExpr::and(vec![
        MetadataFilterExpr::or(vec![
            MetadataFilterExpr::eq("status", "OPEN"),
            MetadataFilterExpr::eq("status", "APPROVED"),
        ]),
        MetadataFilterExpr::between("total_amount", 100, 500),
        MetadataFilterExpr::eq("demo_sale_order.enabled", "Y"),
    ])])
}

pub fn order_export_request() -> MetadataQueryRequest {
    MetadataQueryRequest::new(
        893,
        StatementType::SELECT,
        vec![
            MetadataField::new("demo_sale_order", FieldSource::Column("id".to_string())),
            MetadataField::new("demo_sale_order", FieldSource::Column("code".to_string()))
                .with_access("1")
                .with_sort(SortDirection::Asc)
                .with_output_alias("order_code"),
            MetadataField::new("demo_sale_order", FieldSource::Column("status".to_string()))
                .with_access("1")
                .with_output_alias("status"),
            MetadataField::new("demo_sale_order", FieldSource::Column("total_amount".to_string()))
                .with_access("1")
                .with_output_alias("total_amount"),
            MetadataField::new(
                "demo_sale_order",
                FieldSource::Linked(LinkReference {
                    steps: vec![LinkStep {
                        foreign_key: "store_id".to_string(),
                        table: "demo_store".to_string(),
                    }],
                    target_column: "name".to_string(),
                }),
            )
            .with_access("1")
            .with_output_alias("store_name"),
            MetadataField::new(
                "demo_sale_order",
                FieldSource::Linked(LinkReference {
                    steps: vec![LinkStep {
                        foreign_key: "customer_id".to_string(),
                        table: "demo_customer".to_string(),
                    }],
                    target_column: "name".to_string(),
                }),
            )
            .with_access("1")
            .with_output_alias("customer_name"),
        ],
    )
    .with_options(MetadataQueryOptions {
        table_filter: Some("tenant_id = 37".to_string()),
        ..Default::default()
    })
    .with_filters(vec![MetadataFilterExpr::eq("demo_sale_order.enabled", "Y")])
}

pub fn direct_order_insert_request(
    id: i64,
    code: &str,
    store_id: i64,
    customer_id: i64,
    total_amount: f64,
    status: &str,
) -> MetadataQueryRequest {
    MetadataQueryRequest::new(
        893,
        StatementType::INSERT,
        vec![
            MetadataField::new("demo_sale_order", FieldSource::Column("code".to_string()))
                .with_access("1")
                .with_value(json!(code)),
            MetadataField::new("demo_sale_order", FieldSource::Column("store_id".to_string()))
                .with_access("1")
                .with_value(json!(store_id)),
            MetadataField::new("demo_sale_order", FieldSource::Column("customer_id".to_string()))
                .with_access("1")
                .with_value(json!(customer_id)),
            MetadataField::new("demo_sale_order", FieldSource::Column("status".to_string()))
                .with_access("1")
                .with_value(json!(status)),
            MetadataField::new("demo_sale_order", FieldSource::Column("total_amount".to_string()))
                .with_access("1")
                .with_value(json!(total_amount)),
            MetadataField::new("demo_sale_order", FieldSource::Column("tenant_id".to_string()))
                .with_access("1")
                .with_value(json!(37)),
            MetadataField::new("demo_sale_order", FieldSource::Column("enabled".to_string()))
                .with_access("1")
                .with_value(json!("Y")),
        ],
    )
    .with_options(MetadataQueryOptions {
        id: Some(id),
        ..Default::default()
    })
}

pub fn imported_order_insert_request(
    id: i64,
    code: &str,
    store_name: &str,
    customer_name: &str,
    total_amount: f64,
    status: &str,
) -> MetadataQueryRequest {
    MetadataQueryRequest::new(
        893,
        StatementType::INSERT,
        vec![
            MetadataField::new("demo_sale_order", FieldSource::Column("code".to_string()))
                .with_access("1")
                .with_value(json!(code)),
            MetadataField::new("demo_sale_order", FieldSource::Column("store_id".to_string()))
                .with_access("1")
                .with_lookup("demo_store", "name")
                .with_value(json!(store_name)),
            MetadataField::new("demo_sale_order", FieldSource::Column("customer_id".to_string()))
                .with_access("1")
                .with_lookup("demo_customer", "name")
                .with_value(json!(customer_name)),
            MetadataField::new("demo_sale_order", FieldSource::Column("status".to_string()))
                .with_access("1")
                .with_value(json!(status)),
            MetadataField::new("demo_sale_order", FieldSource::Column("total_amount".to_string()))
                .with_access("1")
                .with_value(json!(total_amount)),
            MetadataField::new("demo_sale_order", FieldSource::Column("tenant_id".to_string()))
                .with_access("1")
                .with_value(json!(37)),
            MetadataField::new("demo_sale_order", FieldSource::Column("enabled".to_string()))
                .with_access("1")
                .with_value(json!("Y")),
        ],
    )
    .with_options(MetadataQueryOptions {
        id: Some(id),
        ..Default::default()
    })
}

pub fn order_update_request(id: i64, total_amount: f64, status: &str) -> MetadataQueryRequest {
    MetadataQueryRequest::new(
        893,
        StatementType::UPDATE,
        vec![
            MetadataField::new("demo_sale_order", FieldSource::Column("status".to_string()))
                .with_access("1")
                .with_value(json!(status)),
            MetadataField::new("demo_sale_order", FieldSource::Column("total_amount".to_string()))
                .with_access("1")
                .with_value(json!(total_amount)),
        ],
    )
    .with_options(MetadataQueryOptions {
        id: Some(id),
        table_filter: Some("tenant_id = 37".to_string()),
        ..Default::default()
    })
}

pub fn order_delete_request(id: i64) -> MetadataQueryRequest {
    MetadataQueryRequest::new(
        893,
        StatementType::DELETE,
        vec![MetadataField::new(
            "demo_sale_order",
            FieldSource::Column("id".to_string()),
        )],
    )
    .with_options(MetadataQueryOptions {
        id: Some(id),
        table_filter: Some("tenant_id = 37".to_string()),
        ..Default::default()
    })
}
