use ora_sql_builder::metadata::{FieldSource, MetadataField, MetadataFilterExpr, MetadataQueryOptions, MetadataQueryRequest};
use ora_sql_builder::metadata_plan::{DeletePlan, QueryPlan, SchemaPlan, WritePlan};
use ora_sql_builder::sql::StatementType;
use serde_json::json;

fn main() {
    let query_request = MetadataQueryRequest::new(
        893,
        StatementType::SELECT,
        vec![
            MetadataField::new("m_retail", FieldSource::Column("id".to_string())),
            MetadataField::new("m_retail", FieldSource::Column("name".to_string()))
                .with_access("1")
                .with_output_alias("name"),
        ],
    )
    .with_options(MetadataQueryOptions {
        table_filter: Some("tenant_id = 37".to_string()),
        ..Default::default()
    })
    .with_filters(vec![MetadataFilterExpr::eq("name", "旗舰店")]);

    let insert_request = MetadataQueryRequest::new(
        893,
        StatementType::INSERT,
        vec![
            MetadataField::new("m_retail", FieldSource::Column("code".to_string()))
                .with_access("1")
                .with_default(json!("RE-001")),
            MetadataField::new("m_retail", FieldSource::Column("docno".to_string()))
                .with_access("1")
                .with_sequence("RE"),
        ],
    )
    .with_options(MetadataQueryOptions {
        id: Some(1),
        ..Default::default()
    });

    let delete_request = MetadataQueryRequest::new(
        893,
        StatementType::DELETE,
        vec![MetadataField::new("m_retail", FieldSource::Column("id".to_string()))],
    )
    .with_options(MetadataQueryOptions {
        id: Some(1),
        table_filter: Some("tenant_id = 37".to_string()),
        ..Default::default()
    })
    .with_filters(vec![MetadataFilterExpr::eq("status", "OPEN")]);

    println!("QueryPlan:\n{:#?}\n", QueryPlan::from_request(&query_request));
    println!("WritePlan:\n{:#?}\n", WritePlan::from_insert_request(&insert_request));
    println!("DeletePlan:\n{:#?}\n", DeletePlan::from_request(&delete_request));
    println!("SchemaPlan:\n{:#?}\n", SchemaPlan::from_standard_metadata());
}
