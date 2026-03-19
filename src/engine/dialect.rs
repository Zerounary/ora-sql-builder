use super::query::Pagination;

pub trait SqlDialect {
    fn placeholder(&self, index: usize) -> String;
    fn paginate(&self, sql: String, pagination: &Pagination, has_order_by: bool) -> String;
}

pub struct MySqlDialect;
pub struct PostgresDialect;
pub struct OracleDialect;
pub struct SqlServerDialect;

impl SqlDialect for MySqlDialect {
    fn placeholder(&self, _index: usize) -> String {
        "?".to_string()
    }

    fn paginate(&self, sql: String, pagination: &Pagination, _has_order_by: bool) -> String {
        format!(
            "{} LIMIT {} OFFSET {}",
            sql, pagination.limit, pagination.offset
        )
    }
}

impl SqlDialect for PostgresDialect {
    fn placeholder(&self, index: usize) -> String {
        format!("${}", index)
    }

    fn paginate(&self, sql: String, pagination: &Pagination, _has_order_by: bool) -> String {
        format!(
            "{} LIMIT {} OFFSET {}",
            sql, pagination.limit, pagination.offset
        )
    }
}

impl SqlDialect for OracleDialect {
    fn placeholder(&self, index: usize) -> String {
        format!(":{}", index)
    }

    fn paginate(&self, sql: String, pagination: &Pagination, _has_order_by: bool) -> String {
        let upper = pagination.offset + pagination.limit;
        format!(
            "SELECT * FROM (SELECT inner_query.*, ROWNUM AS row_num FROM ({}) inner_query WHERE ROWNUM <= {}) WHERE row_num > {}",
            sql, upper, pagination.offset
        )
    }
}

impl SqlDialect for SqlServerDialect {
    fn placeholder(&self, index: usize) -> String {
        format!("@p{}", index)
    }

    fn paginate(&self, sql: String, pagination: &Pagination, has_order_by: bool) -> String {
        let sql = if has_order_by {
            sql
        } else {
            format!("{} ORDER BY (SELECT 1)", sql)
        };
        format!(
            "{} OFFSET {} ROWS FETCH NEXT {} ROWS ONLY",
            sql, pagination.offset, pagination.limit
        )
    }
}
