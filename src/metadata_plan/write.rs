use serde_json::Value;

use crate::metadata::{
    FieldInputKind, MetadataField, MetadataFilterExpr, MetadataId, MetadataQueryRequest,
};
use crate::sql::StatementType;

use super::PermissionPlan;

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
