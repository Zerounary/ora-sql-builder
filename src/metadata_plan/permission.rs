use crate::metadata::{FieldSource, MetadataField, MetadataId, MetadataQueryRequest};

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
