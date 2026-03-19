use serde_json::{json, Value};

use crate::metadata::{standard_metadata_tables, MetadataCatalog};

use super::{
    database_kind_code, filter_expr_text, policy_kind_code, primary_key_strategy_code,
    relation_kind_code, MetadataPersistenceRow, MetadataPersistenceSnapshot,
};

pub struct MetadataPersistenceMapper;

impl MetadataPersistenceMapper {
    pub fn snapshot_from_catalog(catalog: &MetadataCatalog) -> MetadataPersistenceSnapshot {
        let mut rows = Vec::new();

        for datasource in &catalog.datasources {
            rows.push(MetadataPersistenceRow {
                table: "meta_datasource".to_string(),
                values: vec![
                    ("id".to_string(), json!(datasource.id)),
                    ("code".to_string(), json!(datasource.code)),
                    ("name".to_string(), json!(datasource.name)),
                    (
                        "db_type".to_string(),
                        json!(database_kind_code(&datasource.database_kind)),
                    ),
                    ("dsn".to_string(), json!(datasource.connection_uri)),
                    (
                        "default_schema".to_string(),
                        datasource
                            .default_schema
                            .as_ref()
                            .map(|value| json!(value))
                            .unwrap_or(Value::Null),
                    ),
                    ("options".to_string(), datasource.options.clone()),
                    ("enabled".to_string(), json!(datasource.enabled)),
                ],
            });
        }

        for table in &catalog.tables {
            rows.push(MetadataPersistenceRow {
                table: "meta_table".to_string(),
                values: vec![
                    ("id".to_string(), json!(table.id)),
                    ("datasource_id".to_string(), json!(table.datasource_id)),
                    ("table_code".to_string(), json!(table.table_code)),
                    ("table_name".to_string(), json!(table.table_name)),
                    ("display_name".to_string(), json!(table.display_name)),
                    ("enabled".to_string(), json!(table.enabled)),
                    (
                        "primary_key_strategy".to_string(),
                        json!(primary_key_strategy_code(&table.primary_key_strategy)),
                    ),
                    ("logical_delete".to_string(), json!(table.logical_delete)),
                    ("audit_enabled".to_string(), json!(table.audit_enabled)),
                    ("default_sort".to_string(), json!(table.default_sort)),
                ],
            });
        }

        for column in &catalog.columns {
            rows.push(MetadataPersistenceRow {
                table: "meta_column".to_string(),
                values: vec![
                    ("id".to_string(), json!(column.id)),
                    ("table_id".to_string(), json!(column.table_id)),
                    ("column_code".to_string(), json!(column.column_code)),
                    ("column_name".to_string(), json!(column.column_name)),
                    ("display_name".to_string(), json!(column.display_name)),
                    ("data_type".to_string(), json!(column.column_type.sql_type())),
                    ("nullable".to_string(), json!(column.nullable)),
                    ("queryable".to_string(), json!(column.queryable)),
                    ("editable".to_string(), json!(column.editable)),
                    ("sortable".to_string(), json!(column.sortable)),
                    ("primary_key".to_string(), json!(column.primary_key)),
                    (
                        "default_value_sql".to_string(),
                        column
                            .default_value_sql
                            .as_ref()
                            .map(|value| json!(value))
                            .unwrap_or(Value::Null),
                    ),
                ],
            });
        }

        for relation in &catalog.relations {
            rows.push(MetadataPersistenceRow {
                table: "meta_relation".to_string(),
                values: vec![
                    ("id".to_string(), json!(relation.id)),
                    ("left_table_id".to_string(), json!(relation.left_table_id)),
                    ("right_table_id".to_string(), json!(relation.right_table_id)),
                    (
                        "relation_type".to_string(),
                        json!(relation_kind_code(&relation.relation_kind)),
                    ),
                    ("join_type".to_string(), json!(relation.join_type)),
                    ("left_column".to_string(), json!(relation.left_column)),
                    ("right_column".to_string(), json!(relation.right_column)),
                    (
                        "bridge_table".to_string(),
                        relation
                            .bridge_table
                            .as_ref()
                            .map(|value| json!(value))
                            .unwrap_or(Value::Null),
                    ),
                ],
            });
        }

        for policy in &catalog.policies {
            rows.push(MetadataPersistenceRow {
                table: "meta_policy".to_string(),
                values: vec![
                    ("id".to_string(), json!(policy.id)),
                    ("table_id".to_string(), json!(policy.table_id)),
                    ("policy_code".to_string(), json!(policy.policy_code)),
                    (
                        "policy_type".to_string(),
                        json!(policy_kind_code(&policy.policy_kind)),
                    ),
                    (
                        "policy_expr".to_string(),
                        json!(policy.filter.as_ref().map(filter_expr_text)),
                    ),
                    ("enabled".to_string(), json!(policy.enabled)),
                ],
            });
        }

        let mut import_mapping_id: i64 = 1;
        for profile in &catalog.import_profiles {
            rows.push(MetadataPersistenceRow {
                table: "meta_import_profile".to_string(),
                values: vec![
                    ("id".to_string(), json!(profile.id)),
                    ("table_id".to_string(), json!(profile.table_id)),
                    ("profile_code".to_string(), json!(profile.profile_code)),
                    ("display_name".to_string(), json!(profile.display_name)),
                    ("update_keys".to_string(), json!(profile.update_keys)),
                ],
            });
            for mapping in &profile.field_mappings {
                rows.push(MetadataPersistenceRow {
                    table: "meta_import_mapping".to_string(),
                    values: vec![
                        ("id".to_string(), json!(import_mapping_id)),
                        ("profile_id".to_string(), json!(profile.id)),
                        ("source_key".to_string(), json!(mapping.source_key)),
                        (
                            "target_column_code".to_string(),
                            json!(mapping.target_column_code),
                        ),
                        ("required".to_string(), json!(mapping.required)),
                    ],
                });
                import_mapping_id += 1;
            }
        }

        for profile in &catalog.export_profiles {
            rows.push(MetadataPersistenceRow {
                table: "meta_export_profile".to_string(),
                values: vec![
                    ("id".to_string(), json!(profile.id)),
                    ("table_id".to_string(), json!(profile.table_id)),
                    ("profile_code".to_string(), json!(profile.profile_code)),
                    ("display_name".to_string(), json!(profile.display_name)),
                    (
                        "selected_columns".to_string(),
                        json!(profile.selected_column_codes),
                    ),
                    (
                        "default_filter".to_string(),
                        json!(profile.default_filter.as_ref().map(filter_expr_text)),
                    ),
                    ("order_by".to_string(), json!(profile.order_by)),
                ],
            });
        }

        MetadataPersistenceSnapshot {
            schemas: standard_metadata_tables(),
            rows,
        }
    }
}
