use serde_json::json;
use similar_asserts::assert_eq;

use super::*;
use crate::metadata::{
    FieldSource, MetadataField, MetadataFilterExpr, MetadataQueryOptions,
    MetadataQueryRequest, SortDirection,
};
use crate::sql::StatementType;

#[test]
fn query_plan_collects_filters_relations_and_permission() {
    let request = MetadataQueryRequest::new(
        893,
        StatementType::SELECT,
        vec![
            MetadataField::new("m_retail", FieldSource::Column("id".to_string())),
            MetadataField::new("m_retail", FieldSource::Column("name".to_string()))
                .with_access("1")
                .with_sort(SortDirection::Asc)
                .with_output_alias("name"),
        ],
    )
    .with_options(MetadataQueryOptions {
        mask_index: 0,
        table_filter: Some("tenant_id = 37".to_string()),
        ..Default::default()
    })
    .with_filters(vec![MetadataFilterExpr::eq("name", "旗舰店")]);

    let plan = QueryPlan::from_request(&request);

    assert_eq!(plan.table, "m_retail".to_string());
    assert_eq!(plan.table_alias, "m_retail".to_string());
    assert_eq!(plan.filters, vec![MetadataFilterExpr::eq("name", "旗舰店")]);
    assert_eq!(plan.permission.row_filter, Some("tenant_id = 37".to_string()));
    assert_eq!(plan.permission.readable_fields, vec!["name".to_string()]);
}

#[test]
fn write_and_delete_plans_can_be_derived_from_requests() {
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

    let write_plan = WritePlan::from_insert_request(&insert_request);
    let delete_plan = DeletePlan::from_request(&delete_request);

    assert_eq!(write_plan.table, "m_retail".to_string());
    assert_eq!(write_plan.assignments.len(), 2);
    assert_eq!(delete_plan.table, "m_retail".to_string());
    assert_eq!(delete_plan.filters, vec![MetadataFilterExpr::eq("status", "OPEN")]);
}

#[test]
fn schema_plan_loads_standard_metadata_tables() {
    let plan = SchemaPlan::from_standard_metadata();
    assert!(plan.tables.iter().any(|table| table.name == "meta_datasource"));
    assert!(plan.tables.iter().any(|table| table.name == "meta_table"));
    assert!(plan.tables.iter().any(|table| table.name == "meta_column"));
}
