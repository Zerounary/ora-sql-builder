mod builders;
mod dialect;
mod facade;
mod query;

pub use builders::{DeleteBuilder, InsertBuilder, SelectBuilder, UpdateBuilder};
pub use dialect::{MySqlDialect, OracleDialect, PostgresDialect, SqlDialect, SqlServerDialect};
pub use facade::MetaSqlEngine;
pub use query::{BuiltQuery, JoinType, Pagination, Predicate, Relation, TableRef};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use serde_json::Value;
    use similar_asserts::assert_eq;

    #[test]
    fn build_postgres_select_with_relation_predicates_and_pagination() {
        let engine = MetaSqlEngine;
        let dialect = PostgresDialect;
        let query = engine.build_select(
            &dialect,
            SelectBuilder::new(TableRef::new("m_retail").alias("mr"))
                .select("mr.id")
                .select_as("store.name", "store_name")
                .relation(Relation::new(
                    JoinType::Left,
                    "mr",
                    "store_id",
                    TableRef::new("c_store").alias("store"),
                    "id",
                ))
                .predicate(Predicate::eq("mr.owner_id", 893))
                .predicate(Predicate::like("store.name", "%旗舰%"))
                .predicate(Predicate::in_list("mr.status", vec![json!("OPEN"), json!("CLOSED")]))
                .group_by("mr.id")
                .group_by("store.name")
                .order_by("mr.id DESC")
                .paginate(Pagination {
                    offset: 20,
                    limit: 10,
                }),
        );

        assert_eq!(
            query.sql,
            "SELECT mr.id, store.name AS store_name FROM m_retail mr LEFT JOIN c_store store ON mr.store_id = store.id WHERE mr.owner_id = $1 AND store.name LIKE $2 AND mr.status IN ($3, $4) GROUP BY mr.id, store.name ORDER BY mr.id DESC LIMIT 10 OFFSET 20".to_string()
        );
        assert_eq!(query.params, vec![json!(893), json!("%旗舰%"), json!("OPEN"), json!("CLOSED")]);
    }

    #[test]
    fn build_mysql_insert_uses_question_mark_placeholders() {
        let engine = MetaSqlEngine;
        let dialect = MySqlDialect;
        let query = engine.build_insert(
            &dialect,
            InsertBuilder::new("m_retail")
                .value("id", 1)
                .value("code", "RE-001")
                .value("enabled", true),
        );

        assert_eq!(
            query.sql,
            "INSERT INTO m_retail (id, code, enabled) VALUES (?, ?, ?)".to_string()
        );
        assert_eq!(query.params, vec![json!(1), json!("RE-001"), json!(true)]);
    }

    #[test]
    fn build_oracle_update_uses_numbered_placeholders_in_order() {
        let engine = MetaSqlEngine;
        let dialect = OracleDialect;
        let query = engine.build_update(
            &dialect,
            UpdateBuilder::new("m_retail")
                .set("name", "新名称")
                .set("qty", 30)
                .predicate(Predicate::gte("modified_date", "2026-01-01"))
                .predicate(Predicate::eq("id", 1)),
        );

        assert_eq!(
            query.sql,
            "UPDATE m_retail SET name = :1, qty = :2 WHERE modified_date >= :3 AND id = :4".to_string()
        );
        assert_eq!(
            query.params,
            vec![json!("新名称"), json!(30), json!("2026-01-01"), json!(1)]
        );
    }

    #[test]
    fn build_sql_server_delete_adds_fallback_order_by_for_pagination() {
        let dialect = SqlServerDialect;
        let query = SelectBuilder::new(TableRef::new("meta_table"))
            .select("id")
            .predicate(Predicate::is_null("deleted_at"))
            .paginate(Pagination {
                offset: 5,
                limit: 15,
            })
            .build(&dialect);

        assert_eq!(
            query.sql,
            "SELECT id FROM meta_table WHERE deleted_at IS NULL ORDER BY (SELECT 1) OFFSET 5 ROWS FETCH NEXT 15 ROWS ONLY".to_string()
        );
        assert_eq!(query.params, Vec::<Value>::new());
    }

    #[test]
    fn build_delete_with_raw_and_empty_in_list_is_safe() {
        let engine = MetaSqlEngine;
        let dialect = PostgresDialect;
        let query = engine.build_delete(
            &dialect,
            DeleteBuilder::new("meta_operation_log")
                .predicate(Predicate::raw("tenant_id = 37"))
                .predicate(Predicate::in_list("id", Vec::new())),
        );

        assert_eq!(
            query.sql,
            "DELETE FROM meta_operation_log WHERE tenant_id = 37 AND 1 = 0".to_string()
        );
        assert_eq!(query.params, Vec::<Value>::new());
    }

    #[test]
    fn build_sql_server_select_keeps_explicit_order_by_when_paginating() {
        let dialect = SqlServerDialect;
        let query = SelectBuilder::new(TableRef::new("meta_table"))
            .order_by("id DESC")
            .paginate(Pagination {
                offset: 0,
                limit: 5,
            })
            .build(&dialect);

        assert_eq!(
            query.sql,
            "SELECT * FROM meta_table ORDER BY id DESC OFFSET 0 ROWS FETCH NEXT 5 ROWS ONLY"
                .to_string()
        );
        assert_eq!(query.params, Vec::<Value>::new());
    }

    #[test]
    fn build_oracle_select_wraps_paginated_query() {
        let dialect = OracleDialect;
        let query = SelectBuilder::new(TableRef::new("meta_table"))
            .select("id")
            .paginate(Pagination {
                offset: 10,
                limit: 20,
            })
            .build(&dialect);

        assert_eq!(
            query.sql,
            "SELECT * FROM (SELECT inner_query.*, ROWNUM AS row_num FROM (SELECT id FROM meta_table) inner_query WHERE ROWNUM <= 30) WHERE row_num > 10"
                .to_string()
        );
        assert_eq!(query.params, Vec::<Value>::new());
    }

    #[test]
    fn build_inner_join_select_renders_join_keyword() {
        let dialect = PostgresDialect;
        let query = SelectBuilder::new(TableRef::new("m_order").alias("o"))
            .select("o.id")
            .relation(Relation::new(
                JoinType::Inner,
                "o",
                "customer_id",
                TableRef::new("c_customer").alias("c"),
                "id",
            ))
            .build(&dialect);

        assert_eq!(
            query.sql,
            "SELECT o.id FROM m_order o INNER JOIN c_customer c ON o.customer_id = c.id"
                .to_string()
        );
        assert_eq!(query.params, Vec::<Value>::new());
    }
}
