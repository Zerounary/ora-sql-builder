use serde_json::Value;

use super::*;

mod sanitize_filter_value_tests {
    use super::super::helpers::sanitize_filter_value;
    use similar_asserts::assert_eq;

    #[test]
    fn strips_percent_encoded_bytes_from_filter_input() {
        assert_eq!(sanitize_filter_value("%df%5c"), "".to_string());
    }

    #[test]
    fn keeps_regular_filter_input_unchanged() {
        assert_eq!(sanitize_filter_value("旗舰店 A01"), "旗舰店 A01".to_string());
    }
}

mod portal_provider_from {
    use std::time::Instant;

    use super::*;
    use crate::sql::SQLExplorer;
    use serde_json::json;
    use similar_asserts::assert_eq;
    use wildmatch::WildMatch;

    static USER_ID: &str = "893";

    #[test]
    fn empty_select() {
        let data = vec![Column {
            dbname: "id".to_string(),
            current_table: "c_store".to_string(),
            ..Column::default()
        }];
        let portal_privider = PortalProvider::new(USER_ID.parse().unwrap(), data);

        let mut sql_builder = SQLExplorer::new(Box::new(portal_privider));

        let sql = sql_builder.get_sql();

        assert_eq!(sql, "SELECT c_store.id as \"id\"\nFROM c_store".to_string());
    }

    #[test]
    fn single_select() {
        let data = vec![
            Column {
                dbname: "id".to_string(),
                current_table: "c_store".to_string(),
                ..Column::default()
            },
            Column {
                dbname: "name".to_string(),
                mask: "1000000000".to_string(),
                current_table: "c_store".to_string(),
                ..Column::default()
            },
        ];
        let portal_privider = PortalProvider::new_opt(
            USER_ID.parse().unwrap(),
            data,
            PortalProviderOption {
                max_idx: Some(0),
                ..Default::default()
            },
        );

        let mut sql_builder = SQLExplorer::new(Box::new(portal_privider));

        let sql = sql_builder.get_sql();

        assert_eq!(
            sql,
            "SELECT c_store.id as \"id\", c_store.name as \"name\"\nFROM c_store".to_string()
        );
    }

    #[test]
    fn proformance_table_select() {
        let now = Instant::now();
        let count = 10000;
        for _i in 1..count {
            virtual_table_select();
        }
        let end = now.elapsed();
        println!("{}次执行{}ms", count, end.as_millis());
    }

    #[test]
    fn test_where_select() {
        let data = vec![
            Column {
                dbname: "id".to_string(),
                real_table: Some("c_store".to_string()),
                current_table: "v_store".to_string(),
                ..Column::default()
            },
            Column {
                dbname: "name".to_string(),
                mask: "1000000000".to_string(),
                real_table: Some("c_store".to_string()),
                current_table: "v_store".to_string(),
                value: Some(Value::String("%df%5c".to_string())),
                ..Column::default()
            },
        ];
        let portal_privider = PortalProvider::new_opt(
            USER_ID.parse().unwrap(),
            data,
            PortalProviderOption {
                max_idx: Some(0),
                ..Default::default()
            },
        );

        let mut sql_builder = SQLExplorer::new(Box::new(portal_privider));

        let sql = sql_builder.get_sql();

        assert_eq!(
            sql,
            "SELECT v_store.id as \"id\", v_store.name as \"name\"\nFROM c_store v_store\nWHERE (v_store.name = '')"
                .to_string()
        );
    }

    #[test]
    fn virtual_table_select() {
        let data = vec![
            Column {
                dbname: "id".to_string(),
                real_table: Some("c_store".to_string()),
                current_table: "v_store".to_string(),
                ..Column::default()
            },
            Column {
                dbname: "name".to_string(),
                mask: "1000000000".to_string(),
                real_table: Some("c_store".to_string()),
                current_table: "v_store".to_string(),
                ..Column::default()
            },
        ];
        let portal_privider = PortalProvider::new_opt(
            USER_ID.parse().unwrap(),
            data,
            PortalProviderOption {
                max_idx: Some(0),
                ..Default::default()
            },
        );

        let mut sql_builder = SQLExplorer::new(Box::new(portal_privider));

        let sql = sql_builder.get_sql();

        assert_eq!(
            sql,
            "SELECT v_store.id as \"id\", v_store.name as \"name\"\nFROM c_store v_store"
                .to_string()
        );
    }

    #[test]
    fn column_dk_select() {
        let data = vec![
            Column {
                dbname: "id".to_string(),
                current_table: "m_retail".to_string(),
                ..Column::default()
            },
            Column {
                column_id: 123,
                dbname: "C_STORE_ID".to_string(),
                mask: "1000000000".to_string(),
                current_table: "m_retail".to_string(),
                ref_table_id: Some(1),
                ref_table: Some(Dk {
                    table_name: "C_STORE".to_string(),
                    dk_column: "name".to_string(),
                    ..Default::default()
                }),
                obtainmanner: Obtainmanner::Object.to_string(),
                ..Column::default()
            },
        ];
        let portal_privider = PortalProvider::new_opt(
            USER_ID.parse().unwrap(),
            data,
            PortalProviderOption {
                max_idx: Some(0),
                ..Default::default()
            },
        );

        let mut sql_builder = SQLExplorer::new(Box::new(portal_privider));

        let sql = sql_builder.get_sql();
        assert_eq!(
            sql,
            "SELECT m_retail.id as \"id\", m_retail.C_STORE_ID as \"C_STORE_ID\", (select name as dk from C_STORE x where id = m_retail.C_STORE_ID) as \"C_STORE_ID.dk\"\nFROM m_retail".to_string()
        );
    }

    #[test]
    fn column_link_select() {
        let data = vec![
            Column {
                dbname: "id".to_string(),
                current_table: "m_retail".to_string(),
                ..Column::default()
            },
            Column {
                column_id: 123,
                dbname: "C_STORE_ID;C_STOREKIND_ID;NAME".to_string(),
                mask: "1000000000".to_string(),
                current_table: "m_retail".to_string(),
                columnlink_tablenames: vec!["C_STORE".to_string(), "C_STOREKIND".to_string()],
                ..Column::default()
            },
        ];
        let portal_privider = PortalProvider::new_opt(
            USER_ID.parse().unwrap(),
            data,
            PortalProviderOption {
                max_idx: Some(0),
                ..Default::default()
            },
        );

        let mut sql_builder = SQLExplorer::new(Box::new(portal_privider));

        let sql = sql_builder.get_sql();
        assert_eq!(
            sql,
            "SELECT m_retail.id as \"id\", a2.NAME as \"123\"\nFROM m_retail\nLEFT OUTER JOIN C_STORE a1 ON m_retail.C_STORE_ID = a1.id\nLEFT OUTER JOIN C_STOREKIND a2 ON a1.C_STOREKIND_ID = a2.id".to_string()
        );
    }

    #[test]
    fn select_with_boolean_array_and_between_filters() {
        let data = vec![
            Column {
                dbname: "id".to_string(),
                current_table: "m_retail".to_string(),
                ..Column::default()
            },
            Column {
                dbname: "enabled".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                value: Some(json!(true)),
                ..Column::default()
            },
            Column {
                dbname: "status".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                value: Some(json!(["OPEN", "CLOSED"])),
                ..Column::default()
            },
            Column {
                dbname: "amt".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                value: Some(json!({"type": "between", "begin": 10, "end": 20})),
                ..Column::default()
            },
        ];
        let portal_privider = PortalProvider::new_opt(
            USER_ID.parse().unwrap(),
            data,
            PortalProviderOption {
                max_idx: Some(0),
                ..Default::default()
            },
        );

        let mut sql_builder = SQLExplorer::new(Box::new(portal_privider));

        let sql = sql_builder.get_sql();

        assert_eq!(
            sql,
            "SELECT m_retail.id as \"id\", m_retail.enabled as \"enabled\", m_retail.status as \"status\", m_retail.amt as \"amt\"\nFROM m_retail\nWHERE (m_retail.enabled = 'Y' AND m_retail.status IN ('OPEN','CLOSED') AND m_retail.amt >= 10 AND m_retail.amt <= 20)"
                .to_string()
        );
    }

    #[test]
    fn grouped_select_uses_group_by_for_order_by() {
        let data = vec![
            Column {
                dbname: "id".to_string(),
                current_table: "m_retail".to_string(),
                ..Column::default()
            },
            Column {
                dbname: "dept_name".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                ..Column::default()
            },
            Column {
                column_id: 88,
                dbname: "sum(qty)".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                ..Column::default()
            },
        ];
        let portal_privider = PortalProvider::new_opt(
            USER_ID.parse().unwrap(),
            data,
            PortalProviderOption {
                max_idx: Some(0),
                is_group: Some(true),
                ..Default::default()
            },
        );

        let mut sql_builder = SQLExplorer::new(Box::new(portal_privider));

        let sql = sql_builder.get_sql();

        assert_eq!(
            sql,
            "SELECT m_retail.dept_name as \"dept_name\", sum(qty) as \"88\"\nFROM m_retail\nGROUP BY m_retail.dept_name\nORDER BY m_retail.dept_name"
                .to_string()
        );
    }

    #[test]
    fn common_update() {
        let data = vec![
            Column {
                dbname: "name".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                value: Some(Value::String("名称".to_string())),
                ..Column::default()
            },
            Column {
                dbname: "qty".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                value: Some(json!(30)),
                ..Column::default()
            },
            Column {
                dbname: "amt".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                value: Some(json!(30.13)),
                ..Column::default()
            },
        ];
        let mut portal_privider = PortalProvider::new_opt(
            USER_ID.parse().unwrap(),
            data,
            PortalProviderOption {
                id: Some(1),
                max_idx: Some(0),
                ..Default::default()
            },
        );

        portal_privider.statement_type(StatementType::UPDATE);

        let mut sql_builder = SQLExplorer::new(Box::new(portal_privider));

        let sql = sql_builder.get_sql();

        assert_eq!(
            sql,
            "UPDATE m_retail\nSET modifierid=893, modifieddate = sysdate, name = '名称', qty = 30, amt = 30.13\nWHERE (id = 1)".to_string()
        );
    }

    #[test]
    fn common_insert() {
        let data = vec![
            Column {
                dbname: "code".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                default_value: "默认值".to_string(),
                ..Column::default()
            },
            Column {
                dbname: "name".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                value: Some(Value::String("名称".to_string())),
                ..Column::default()
            },
            Column {
                dbname: "qty".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                value: Some(json!(30)),
                ..Column::default()
            },
            Column {
                dbname: "amt".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                value: Some(json!(30.13)),
                ..Column::default()
            },
            Column {
                dbname: "docno".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                value: Some(json!("")),
                obtainmanner: Obtainmanner::SheetNo.to_string(),
                sequencename: "RE".to_string(),
                ..Column::default()
            },
            Column {
                dbname: "C_CUSTOMER_ID".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                ref_table_id: Some(1),
                ref_table: Some(Dk {
                    table_name: "c_store".to_string(),
                    dk_column: "name".to_string(),
                    ..Default::default()
                }),
                value: Some(json!(13)),
                obtainmanner: Obtainmanner::Object.to_string(),
                ..Column::default()
            },
            Column {
                dbname: "C_VIP_ID".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                ref_table_id: Some(1),
                ref_table: Some(Dk {
                    table_name: "c_store".to_string(),
                    dk_column: "name".to_string(),
                    ..Default::default()
                }),
                value: Some(json!("12")),
                obtainmanner: Obtainmanner::Object.to_string(),
                ..Column::default()
            },
            Column {
                dbname: "C_STORE_ID".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                ref_table_id: Some(1),
                ref_table: Some(Dk {
                    table_name: "c_store".to_string(),
                    dk_column: "name".to_string(),
                    ..Default::default()
                }),
                value: Some(json!("一号店")),
                obtainmanner: Obtainmanner::Object.to_string(),
                ..Column::default()
            },
            Column {
                dbname: "ROUND(QTYIN - QTYOUT, 0)".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                value: Some(json!("差异数量")),
                obtainmanner: Obtainmanner::Object.to_string(),
                ..Column::default()
            },
        ];
        let mut portal_privider = PortalProvider::new_opt(
            USER_ID.parse().unwrap(),
            data,
            PortalProviderOption {
                id: Some(1),
                max_idx: Some(0),
                ..Default::default()
            },
        );

        portal_privider.statement_type(StatementType::INSERT);

        let mut sql_builder = SQLExplorer::new(Box::new(portal_privider));

        let sql = sql_builder.get_sql();

        assert_eq!(
            sql,
            "INSERT INTO m_retail\n (id, ad_client_id, ad_org_id, ownerid, modifiered, creationdate, modifieddate, code, name, qty, amt, docno, C_CUSTOMER_ID, C_VIP_ID, C_STORE_ID)\nVALUES (1, 37, 27, 893, 893, sysdate, sysdate, '默认值', '名称', 30, 30.13, get_sequenceno('RE', 37), 13, 12, (select id from c_store where name = '一号店'))".to_string()
        );
    }

    #[test]
    fn test_json() {
        let v = serde_json::json!({"a": 1, "a.b": 2});

        println!("{}", v.to_string());
    }

    #[test]
    fn test_regex() {
        let _count = 10000;
        let dbname = "description( nvl(t.tot_qty,0 ";
        let is_match = !dbname.contains([' ', '(']);
        println!("{}", is_match);
        let now = Instant::now();
        for _i in 0..1 {
            if WildMatch::new("*(*)").matches(dbname) {
                println!("{}", dbname);
            }
        }
        println!("{}", now.elapsed().as_millis());
    }
}
