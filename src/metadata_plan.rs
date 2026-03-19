use serde_json::Value;

use crate::metadata::{
    standard_metadata_tables, FieldInputKind, FieldSource, LinkReference, LookupReference,
    MetaColumn, MetaDatasource, MetaExportProfile, MetaImportProfile, MetaPolicy, MetaRelation,
    MetaTable, MetadataField, MetadataFilterExpr, MetadataId, MetadataQueryRequest,
    MetadataTableSchema, SortDirection,
};
use crate::sql::StatementType;

#[derive(Debug, Clone, PartialEq)]
pub struct PermissionPlan {
    pub user_id: MetadataId,
    pub mask_index: usize,
    pub row_filter: Option<String>,
    pub readable_fields: Vec<String>,
    pub writable_fields: Vec<String>,
}

impl PermissionPlan {
    pub fn from_request(request: &MetadataQueryRequest) -> Self {
        let readable_fields = request
            .fields
            .iter()
            .filter(|field| field.access.allows(request.options.mask_index))
            .map(MetadataField::output_name)
            .collect();
        let writable_fields = request
            .fields
            .iter()
            .filter(|field| {
                field.access.allows(request.options.mask_index)
                    && matches!(field.source, FieldSource::Column(_))
            })
            .filter_map(|field| field.source_column().map(ToString::to_string))
            .collect();

        Self {
            user_id: request.user_id,
            mask_index: request.options.mask_index,
            row_filter: request.options.table_filter.clone(),
            readable_fields,
            writable_fields,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectionPlan {
    pub output_name: String,
    pub source: FieldSource,
    pub lookup: Option<LookupReference>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortPlan {
    pub field_name: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationPlan {
    pub output_name: String,
    pub link: LinkReference,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryPlan {
    pub source_request: MetadataQueryRequest,
    pub table: String,
    pub table_alias: String,
    pub target_id: Option<MetadataId>,
    pub grouped: bool,
    pub projections: Vec<ProjectionPlan>,
    pub relations: Vec<RelationPlan>,
    pub filters: Vec<MetadataFilterExpr>,
    pub having: Vec<MetadataFilterExpr>,
    pub sorts: Vec<SortPlan>,
    pub permission: PermissionPlan,
}

impl QueryPlan {
    pub fn from_request(request: &MetadataQueryRequest) -> Self {
        let main_field = request
            .fields
            .first()
            .expect("metadata request requires at least one field");
        let projections: Vec<ProjectionPlan> = request
            .fields
            .iter()
            .filter(|field| field.access.allows(request.options.mask_index))
            .map(|field| ProjectionPlan {
                output_name: field.output_name(),
                source: field.source.clone(),
                lookup: field.lookup.clone(),
            })
            .collect();
        let relations = request
            .fields
            .iter()
            .filter_map(|field| match &field.source {
                FieldSource::Linked(link) if field.access.allows(request.options.mask_index) => {
                    Some(RelationPlan {
                        output_name: field.output_name(),
                        link: link.clone(),
                    })
                }
                _ => None,
            })
            .collect();
        let sorts = request
            .fields
            .iter()
            .filter_map(|field| {
                field.sort.as_ref().map(|direction| SortPlan {
                    field_name: field.output_name(),
                    direction: direction.clone(),
                })
            })
            .collect();

        Self {
            source_request: request.clone(),
            table: main_field
                .real_table
                .clone()
                .unwrap_or_else(|| main_field.current_table.clone()),
            table_alias: main_field.current_table.clone(),
            target_id: request.options.id,
            grouped: request.options.grouped,
            projections,
            relations,
            filters: request.filters.clone(),
            having: request.having.clone(),
            sorts,
            permission: PermissionPlan::from_request(request),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WriteValuePlan {
    Param(Value),
    Default(Value),
    Lookup {
        table: String,
        display_column: String,
        value: Value,
    },
    Sequence(String),
    Raw(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WriteAssignmentPlan {
    pub column_name: String,
    pub value: WriteValuePlan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WritePlan {
    pub source_request: MetadataQueryRequest,
    pub statement_type: StatementType,
    pub table: String,
    pub target_id: Option<MetadataId>,
    pub assignments: Vec<WriteAssignmentPlan>,
    pub permission: PermissionPlan,
}

impl WritePlan {
    pub fn from_insert_request(request: &MetadataQueryRequest) -> Self {
        Self::from_request(request, StatementType::INSERT)
    }

    pub fn from_update_request(request: &MetadataQueryRequest) -> Self {
        Self::from_request(request, StatementType::UPDATE)
    }

    fn from_request(request: &MetadataQueryRequest, expected: StatementType) -> Self {
        let main_field = request
            .fields
            .first()
            .expect("metadata request requires at least one field");
        let assignments = request
            .fields
            .iter()
            .filter_map(write_assignment_from_field)
            .collect();

        Self {
            source_request: request.clone(),
            statement_type: expected,
            table: main_field
                .real_table
                .clone()
                .unwrap_or_else(|| main_field.current_table.clone()),
            target_id: request.options.id,
            assignments,
            permission: PermissionPlan::from_request(request),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeletePlan {
    pub source_request: MetadataQueryRequest,
    pub table: String,
    pub target_id: Option<MetadataId>,
    pub filters: Vec<MetadataFilterExpr>,
    pub permission: PermissionPlan,
}

impl DeletePlan {
    pub fn from_request(request: &MetadataQueryRequest) -> Self {
        let main_field = request
            .fields
            .first()
            .expect("metadata request requires at least one field");
        Self {
            source_request: request.clone(),
            table: main_field
                .real_table
                .clone()
                .unwrap_or_else(|| main_field.current_table.clone()),
            target_id: request.options.id,
            filters: request.filters.clone(),
            permission: PermissionPlan::from_request(request),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaPlan {
    pub tables: Vec<MetadataTableSchema>,
}

impl SchemaPlan {
    pub fn from_standard_metadata() -> Self {
        Self {
            tables: standard_metadata_tables(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetadataRuntimeModel {
    pub datasources: Vec<MetaDatasource>,
    pub tables: Vec<MetaTable>,
    pub columns: Vec<MetaColumn>,
    pub relations: Vec<MetaRelation>,
    pub policies: Vec<MetaPolicy>,
    pub import_profiles: Vec<MetaImportProfile>,
    pub export_profiles: Vec<MetaExportProfile>,
}

impl MetadataRuntimeModel {
    pub fn new() -> Self {
        Self {
            datasources: Vec::new(),
            tables: Vec::new(),
            columns: Vec::new(),
            relations: Vec::new(),
            policies: Vec::new(),
            import_profiles: Vec::new(),
            export_profiles: Vec::new(),
        }
    }
}

fn write_assignment_from_field(field: &MetadataField) -> Option<WriteAssignmentPlan> {
    let column_name = field.source_column()?.to_string();
    let value = match field.input_kind {
        FieldInputKind::Sequence => {
            WriteValuePlan::Sequence(field.sequence_name.clone().unwrap_or_default())
        }
        FieldInputKind::Lookup => {
            let lookup = field.lookup.as_ref()?;
            WriteValuePlan::Lookup {
                table: lookup.table.clone(),
                display_column: lookup.display_column.clone(),
                value: field
                    .value
                    .clone()
                    .or_else(|| field.default_value.clone())
                    .unwrap_or(Value::Null),
            }
        }
        _ => {
            if let Some(value) = &field.value {
                WriteValuePlan::Param(value.clone())
            } else if let Some(default_value) = &field.default_value {
                WriteValuePlan::Default(default_value.clone())
            } else {
                return None;
            }
        }
    };

    Some(WriteAssignmentPlan { column_name, value })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::{FieldSource, MetadataFilterExpr, MetadataQueryOptions};
    use serde_json::json;
    use similar_asserts::assert_eq;

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
}
