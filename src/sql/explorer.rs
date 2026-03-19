use super::{SQLProvider, SQLStatement};

fn populate_statement(sql: &mut SQLStatement, sql_provider: &mut dyn SQLProvider) {
    sql.statement_type = sql_provider.get_statement_type();
    sql.tables = sql_provider.get_tables();
    sql.select = sql_provider.get_select();
    sql.join = sql_provider.get_join();
    sql.inner_join = sql_provider.get_inner_join();
    sql.outer_join = sql_provider.get_outer_join();
    sql.left_outer_join = sql_provider.get_left_outer_join();
    sql.right_outer_join = sql_provider.get_right_outer_join();
    sql.wheres = sql_provider.get_where();
    sql.group_by = sql_provider.get_group_by();
    sql.having = sql_provider.get_having();
    sql.order_by = sql_provider.get_order_by();
    sql.columns = sql_provider.get_columns();
    sql.values = sql_provider.get_values();
    sql.sets = sql_provider.get_sets();
}

pub struct SQLExplorer {
    sql: SQLStatement,
    sql_provider: Box<dyn SQLProvider>,
}

impl SQLExplorer {
    pub fn new(sql_provider: Box<dyn SQLProvider>) -> SQLExplorer {
        SQLExplorer {
            sql: SQLStatement::new(sql_provider.get_statement_type()),
            sql_provider,
        }
    }

    pub fn get_sql(&mut self) -> String {
        populate_statement(&mut self.sql, self.sql_provider.as_mut());
        self.sql.sql()
    }
}

pub fn get_sql(sql_provider: &mut impl SQLProvider) -> String {
    let mut sql = SQLStatement::new(sql_provider.get_statement_type());
    populate_statement(&mut sql, sql_provider);
    sql.sql()
}
