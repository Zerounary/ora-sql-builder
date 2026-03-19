use crate::metadata::{
    FieldSource, LinkReference, LookupReference, MetadataFilterExpr, MetadataId,
    MetadataQueryRequest, SortDirection,
};

use super::PermissionPlan;

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
