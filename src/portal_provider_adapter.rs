use serde_json::Value;
use wildmatch::WildMatch;

use crate::metadata::{
    FieldInputKind, FieldSource, LinkReference, LinkStep, LookupReference, MetadataField,
    MetadataFilterExpr, MetadataQueryOptions, MetadataQueryRequest, SortDirection,
};
use crate::portal_provider::{Column, Obtainmanner, PortalProviderOption};
use crate::sql::StatementType;

pub fn portal_to_metadata_request(
    user_id: i64,
    statement_type: StatementType,
    columns: Vec<Column>,
    args: PortalProviderOption,
) -> MetadataQueryRequest {
    let mut fields: Vec<MetadataField> = columns
        .into_iter()
        .map(|column| metadata_field_from_portal_column(&column, statement_type))
        .collect();

    let filters = if statement_type == StatementType::SELECT {
        let filters = fields
            .iter()
            .filter_map(metadata_filter_from_field)
            .collect::<Vec<_>>();
        for field in &mut fields {
            field.value = None;
        }
        filters
    } else {
        Vec::new()
    };

    MetadataQueryRequest::new(user_id, statement_type, fields)
        .with_options(MetadataQueryOptions {
            id: args.id,
            mask_index: args.max_idx.unwrap_or_default(),
            grouped: args.is_group.unwrap_or(false),
            table_filter: args.table_filter,
            ..Default::default()
        })
        .with_filters(filters)
}

pub fn metadata_field_from_portal_column(column: &Column, statement_type: StatementType) -> MetadataField {
    let mut field = MetadataField::new(column.current_table.clone(), source_from_column(column));
    field.field_id = column.column_id;
    field.real_table = column.real_table.clone();
    field.access = column.mask.clone().into();
    field.nullable = column.nullable;
    field.input_kind = input_kind_from_obtainmanner(column.obtainmanner.as_str(), column);
    field.sequence_name = if column.sequencename.is_empty() {
        None
    } else {
        Some(column.sequencename.clone())
    };
    field.default_value = if column.default_value.is_empty() {
        None
    } else {
        Some(Value::String(column.default_value.clone()))
    };
    field.value = if statement_type == StatementType::SELECT {
        column.value.clone()
    } else {
        column.value.clone()
    };
    field.sort = sort_from_order_by(column.order_by.as_str());
    field.lookup = column.ref_table.as_ref().map(|dk| LookupReference {
        table: dk.table_name.clone(),
        display_column: dk.dk_column.clone(),
    });
    field
}

fn metadata_filter_from_field(field: &MetadataField) -> Option<MetadataFilterExpr> {
    let target = field
        .output_alias
        .clone()
        .unwrap_or_else(|| field.output_name());
    match field.value.as_ref()? {
        Value::Null => None,
        Value::Bool(value) => Some(MetadataFilterExpr::eq(target, *value)),
        Value::Number(value) => Some(MetadataFilterExpr::eq(target, value.clone())),
        Value::String(value) => {
            if value.starts_with('=') {
                Some(MetadataFilterExpr::eq(target, value.trim_start_matches('=')))
            } else if value.contains(' ') {
                Some(MetadataFilterExpr::or(
                    value
                        .split_whitespace()
                        .map(|term| MetadataFilterExpr::like(target.clone(), format!("%{}%", term.trim())))
                        .collect(),
                ))
            } else {
                Some(MetadataFilterExpr::eq(target, value.clone()))
            }
        }
        Value::Array(values) => Some(MetadataFilterExpr::in_list(target, values.clone())),
        Value::Object(object) => {
            if object.get("type") == Some(&Value::String("between".to_string())) {
                Some(MetadataFilterExpr::between(
                    target,
                    object.get("begin").cloned().unwrap_or(Value::Null),
                    object.get("end").cloned().unwrap_or(Value::Null),
                ))
            } else {
                None
            }
        }
    }
}

fn source_from_column(column: &Column) -> FieldSource {
    if column.dbname.contains(';') {
        let parts: Vec<&str> = column.dbname.split(';').collect();
        let target_column = parts.last().unwrap_or(&column.dbname.as_str()).to_string();
        let steps = column
            .columnlink_tablenames
            .iter()
            .zip(parts.iter())
            .map(|(table, foreign_key)| LinkStep {
                foreign_key: (*foreign_key).to_string(),
                table: table.clone(),
            })
            .collect();
        return FieldSource::Linked(LinkReference { steps, target_column });
    }

    if WildMatch::new("*(*)*").matches(&column.dbname) || column.dbname.contains(' ') {
        return FieldSource::Formula(column.dbname.clone());
    }

    if column.dbname.contains('.') {
        return FieldSource::Qualified(column.dbname.clone());
    }

    FieldSource::Column(column.dbname.clone())
}

fn input_kind_from_obtainmanner(obtainmanner: &str, column: &Column) -> FieldInputKind {
    match Obtainmanner::from(obtainmanner) {
        Obtainmanner::Text => FieldInputKind::Text,
        Obtainmanner::Object => {
            if column.ref_table.is_some() {
                FieldInputKind::Lookup
            } else {
                FieldInputKind::Text
            }
        }
        Obtainmanner::Ignore => FieldInputKind::Ignored,
        Obtainmanner::Operate => FieldInputKind::Operation,
        Obtainmanner::SheetNo => FieldInputKind::Sequence,
        Obtainmanner::Triger => FieldInputKind::Trigger,
    }
}

fn sort_from_order_by(order_by: &str) -> Option<SortDirection> {
    match order_by {
        "+" => Some(SortDirection::Asc),
        "-" => Some(SortDirection::Desc),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::PostgresDialect;
    use crate::metadata_driver::MetadataSqlDriver;
    use serde_json::json;
    use similar_asserts::assert_eq;

    #[test]
    fn portal_select_can_be_adapted_to_metadata_request() {
        let request = portal_to_metadata_request(
            893,
            StatementType::SELECT,
            vec![
                Column {
                    dbname: "id".to_string(),
                    current_table: "m_retail".to_string(),
                    ..Column::default()
                },
                Column {
                    dbname: "name".to_string(),
                    mask: "1".to_string(),
                    current_table: "m_retail".to_string(),
                    value: Some(json!("旗舰 店")),
                    order_by: "+".to_string(),
                    ..Column::default()
                },
                Column {
                    dbname: "enabled".to_string(),
                    mask: "1".to_string(),
                    current_table: "m_retail".to_string(),
                    value: Some(json!(true)),
                    ..Column::default()
                },
            ],
            PortalProviderOption {
                max_idx: Some(0),
                table_filter: Some("tenant_id = 37".to_string()),
                ..Default::default()
            },
        );

        let query = MetadataSqlDriver::new(request).build(&PostgresDialect);

        assert_eq!(
            query.sql,
            "SELECT m_retail.id AS \"id\", m_retail.name AS \"name\", m_retail.enabled AS \"enabled\" FROM m_retail WHERE tenant_id = 37 AND (m_retail.name LIKE $1 OR m_retail.name LIKE $2) AND m_retail.enabled = $3 ORDER BY m_retail.name asc nulls first".to_string()
        );
        assert_eq!(query.params, vec![json!("%旗舰%"), json!("%店%"), json!(true)]);
    }

    #[test]
    fn portal_insert_can_be_adapted_to_metadata_request() {
        let request = portal_to_metadata_request(
            893,
            StatementType::INSERT,
            vec![
                Column {
                    dbname: "code".to_string(),
                    mask: "1".to_string(),
                    current_table: "m_retail".to_string(),
                    default_value: "默认值".to_string(),
                    ..Column::default()
                },
                Column {
                    dbname: "docno".to_string(),
                    mask: "1".to_string(),
                    current_table: "m_retail".to_string(),
                    sequencename: "RE".to_string(),
                    obtainmanner: Obtainmanner::SheetNo.to_string(),
                    ..Column::default()
                },
            ],
            PortalProviderOption {
                id: Some(1),
                max_idx: Some(0),
                ..Default::default()
            },
        );

        let query = MetadataSqlDriver::new(request).build(&PostgresDialect);

        assert_eq!(
            query.sql,
            "INSERT INTO m_retail (id, ad_client_id, ad_org_id, ownerid, modifiered, creationdate, modifieddate, code, docno) VALUES ($1, $2, $3, $4, $5, sysdate, sysdate, $6, get_sequenceno('RE', 37))".to_string()
        );
        assert_eq!(
            query.params,
            vec![json!(1), json!(37), json!(27), json!(893), json!(893), json!("默认值")]
        );
    }
}
