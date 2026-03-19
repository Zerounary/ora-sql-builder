use similar_asserts::assert_eq;

use super::*;
use super::statement::{AND, OR};

#[test]
fn build_query_test() {
    let mut sql = SQLStatement::new(StatementType::SELECT);
    sql.select.push("level".to_string());
    sql.select.push("count(1) as num".to_string());
    sql.tables.push("c_store".to_string());
    sql.wheres.push("type=1".to_string());
    sql.group_by.push("level".to_string());
    sql.having.push("count(1) > 2".to_string());
    sql.order_by.push("level asc".to_string());
    let rst = sql.sql();
    assert_eq!(rst, "SELECT level, count(1) as num\nFROM c_store\nWHERE (type=1)\nGROUP BY level\nHAVING (count(1) > 2)\nORDER BY level asc".to_string());
}

#[test]
fn build_insert_test() {
    let mut sql = SQLStatement::new(StatementType::INSERT);
    sql.tables.push("c_store".to_string());
    sql.columns.push("id".to_string());
    sql.columns.push("code".to_string());
    sql.columns.push("name".to_string());
    sql.values.push("1".to_string());
    sql.values.push("'001'".to_string());
    sql.values.push("'Big Star'".to_string());
    let rst = sql.sql();
    assert_eq!(
        rst,
        "INSERT INTO c_store\n (id, code, name)\nVALUES (1, '001', 'Big Star')".to_string()
    );
}

#[test]
fn build_update_test() {
    let mut sql = SQLStatement::new(StatementType::UPDATE);
    sql.tables.push("c_store".to_string());
    sql.sets.push("code = '002'".to_string());
    sql.sets.push("name = 'Big Sun'".to_string());
    sql.wheres.push("id = 1".to_string());
    let rst = sql.sql();
    assert_eq!(
        rst,
        "UPDATE c_store\nSET code = '002', name = 'Big Sun'\nWHERE (id = 1)".to_string()
    );
}

#[test]
fn build_delete_test() {
    let mut sql = SQLStatement::new(StatementType::DELETE);
    sql.tables.push("c_store".to_string());
    sql.wheres.push("id = 1".to_string());
    let rst = sql.sql();
    assert_eq!(rst, "DELETE FROM c_store\nWHERE (id = 1)".to_string());
}

#[test]
fn build_where_clause_preserves_explicit_logical_group_markers() {
    let mut sql = SQLStatement::new(StatementType::SELECT);
    sql.select.push("id".to_string());
    sql.tables.push("c_store".to_string());
    sql.wheres.push("type = 1".to_string());
    sql.wheres.push(OR.to_string());
    sql.wheres.push("type = 2".to_string());
    sql.wheres.push(AND.to_string());
    sql.wheres.push("enabled = 'Y'".to_string());

    let rst = sql.sql();

    assert_eq!(
        rst,
        "SELECT id\nFROM c_store\nWHERE (type = 1) \n OR (type = 2) \n AND (enabled = 'Y')"
            .to_string()
    );
}

#[test]
fn build_select_with_left_outer_join_test() {
    let mut sql = SQLStatement::new(StatementType::SELECT);
    sql.select.push("m_retail.id".to_string());
    sql.tables.push("m_retail".to_string());
    sql.left_outer_join
        .push("c_store s ON m_retail.c_store_id = s.id".to_string());

    let rst = sql.sql();

    assert_eq!(
        rst,
        "SELECT m_retail.id\nFROM m_retail\nLEFT OUTER JOIN c_store s ON m_retail.c_store_id = s.id"
            .to_string()
    );
}
