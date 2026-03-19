mod context;
mod driver;
mod filters;
mod helpers;

pub use driver::MetadataSqlDriver;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::PostgresDialect;
    use crate::metadata::{
        FieldSource, LinkReference, LinkStep, MetadataField, MetadataQueryOptions,
        MetadataQueryRequest, SortDirection,
    };
    use crate::sql::StatementType;
    use serde_json::{json, Value};
    use similar_asserts::assert_eq;

    #[test]
    fn select_with_lookup_and_link_fields_builds_expected_sql() {
        let request = MetadataQueryRequest::new(
            893,
            StatementType::SELECT,
            vec![
                MetadataField::new("m_retail", FieldSource::Column("id".to_string())),
                MetadataField::new("m_retail", FieldSource::Column("name".to_string()))
                    .with_access("1")
                    .with_output_alias("name"),
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
        );

        let query = MetadataSqlDriver::new(request).build(&PostgresDialect);

        assert_eq!(
            query.sql,
            "SELECT m_retail.id AS \"id\", m_retail.name AS \"name\", m_retail.store_id AS \"store_id\", (select name as dk from c_store x where id = m_retail.store_id) as \"store_id.dk\", a2.name AS \"store_kind_name\" FROM m_retail LEFT JOIN c_store a1 ON m_retail.store_id = a1.id LEFT JOIN c_store_kind a2 ON a1.store_kind_id = a2.id".to_string()
        );
        assert_eq!(query.params, Vec::<Value>::new());
    }

    #[test]
    fn select_filters_are_parameterized() {
        let request = MetadataQueryRequest::new(
            893,
            StatementType::SELECT,
            vec![
                MetadataField::new("m_retail", FieldSource::Column("id".to_string())),
                MetadataField::new("m_retail", FieldSource::Column("name".to_string()))
                    .with_access("1")
                    .with_value(json!("旗舰 店"))
                    .with_sort(SortDirection::Asc),
                MetadataField::new("m_retail", FieldSource::Column("enabled".to_string()))
                    .with_access("1")
                    .with_value(json!(true)),
                MetadataField::new("m_retail", FieldSource::Column("status".to_string()))
                    .with_access("1")
                    .with_value(json!(["OPEN", "CLOSED"])),
                MetadataField::new("m_retail", FieldSource::Column("amt".to_string()))
                    .with_access("1")
                    .with_value(json!({"type": "between", "begin": 10, "end": 20})),
            ],
        )
        .with_options(MetadataQueryOptions {
            table_filter: Some("tenant_id = 37".to_string()),
            ..Default::default()
        });

        let query = MetadataSqlDriver::new(request).build(&PostgresDialect);

        assert_eq!(
            query.sql,
            "SELECT m_retail.id AS \"id\", m_retail.name AS \"name\", m_retail.enabled AS \"enabled\", m_retail.status AS \"status\", m_retail.amt AS \"amt\" FROM m_retail WHERE tenant_id = 37 AND (m_retail.name LIKE $1 OR m_retail.name LIKE $2) AND m_retail.enabled = $3 AND m_retail.status IN ($4, $5) AND m_retail.amt >= $6 AND m_retail.amt <= $7 ORDER BY m_retail.name asc nulls first".to_string()
        );
        assert_eq!(
            query.params,
            vec![
                json!("%旗舰%"),
                json!("%店%"),
                json!("Y"),
                json!("OPEN"),
                json!("CLOSED"),
                json!(10),
                json!(20),
            ]
        );
    }

    #[test]
    fn grouped_select_orders_by_grouped_dimensions() {
        let request = MetadataQueryRequest::new(
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
        });

        let query = MetadataSqlDriver::new(request).build(&PostgresDialect);

        assert_eq!(
            query.sql,
            "SELECT m_retail.dept_name AS \"dept_name\", sum(qty) AS \"total_qty\" FROM m_retail GROUP BY m_retail.dept_name ORDER BY m_retail.dept_name".to_string()
        );
        assert_eq!(query.params, Vec::<Value>::new());
    }

    #[test]
    fn insert_uses_parameters_and_raw_sql_expressions() {
        let request = MetadataQueryRequest::new(
            893,
            StatementType::INSERT,
            vec![
                MetadataField::new("m_retail", FieldSource::Column("code".to_string()))
                    .with_access("1")
                    .with_default(json!("默认值")),
                MetadataField::new("m_retail", FieldSource::Column("name".to_string()))
                    .with_access("1")
                    .with_value(json!("名称")),
                MetadataField::new("m_retail", FieldSource::Column("qty".to_string()))
                    .with_access("1")
                    .with_value(json!(30)),
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

        let query = MetadataSqlDriver::new(request).build(&PostgresDialect);

        assert_eq!(
            query.sql,
            "INSERT INTO m_retail (id, ad_client_id, ad_org_id, ownerid, modifiered, creationdate, modifieddate, code, name, qty, docno, customer_id) VALUES ($1, $2, $3, $4, $5, sysdate, sysdate, $6, $7, $8, get_sequenceno('RE', 37), (select id from c_store where name = '一号店'))".to_string()
        );
        assert_eq!(
            query.params,
            vec![
                json!(1),
                json!(37),
                json!(27),
                json!(893),
                json!(893),
                json!("默认值"),
                json!("名称"),
                json!(30),
            ]
        );
    }

    #[test]
    fn update_uses_mixed_parameter_and_raw_assignments() {
        let request = MetadataQueryRequest::new(
            893,
            StatementType::UPDATE,
            vec![
                MetadataField::new("m_retail", FieldSource::Column("name".to_string()))
                    .with_access("1")
                    .with_value(json!("新名称")),
                MetadataField::new("m_retail", FieldSource::Column("store_id".to_string()))
                    .with_access("1")
                    .with_lookup("c_store", "name")
                    .with_value(json!("一号店")),
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

        let query = MetadataSqlDriver::new(request).build(&PostgresDialect);

        assert_eq!(
            query.sql,
            "UPDATE m_retail SET modifierid = $1, modifieddate = sysdate, name = $2, store_id = (select id from c_store where name = '一号店'), enabled = $3 WHERE tenant_id = 37 AND id = $4".to_string()
        );
        assert_eq!(
            query.params,
            vec![json!(893), json!("新名称"), json!("Y"), json!(1)]
        );
    }

    #[test]
    fn delete_uses_table_filter_and_id_predicate() {
        let request = MetadataQueryRequest::new(893, StatementType::DELETE, vec![
            MetadataField::new("m_retail", FieldSource::Column("id".to_string())),
        ])
        .with_options(MetadataQueryOptions {
            id: Some(9),
            table_filter: Some("tenant_id = 37".to_string()),
            ..Default::default()
        });

        let query = MetadataSqlDriver::new(request).build(&PostgresDialect);

        assert_eq!(
            query.sql,
            "DELETE FROM m_retail WHERE tenant_id = 37 AND id = $1".to_string()
        );
        assert_eq!(query.params, vec![json!(9)]);
    }
}
