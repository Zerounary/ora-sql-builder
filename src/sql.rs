static AND: &'static str = ") \n AND (";
static OR: &'static str = ") \n OR (";
pub struct SQLExplorer {
    sql: SQLStatement,
    sql_provider: Box<dyn SQLProvider>,
}

pub trait SQLProvider {
    fn get_statement_type(&self) -> StatementType;
    fn get_select(&mut self) -> Vec<String>;
    fn get_tables(&mut self) -> Vec<String>;
    fn get_join(&self) -> Vec<String>;
    fn get_inner_join(&self) -> Vec<String>;
    fn get_outer_join(&self) -> Vec<String>;
    fn get_left_outer_join(&mut self) -> Vec<String>;
    fn get_right_outer_join(&self) -> Vec<String>;
    fn get_where(&mut self) -> Vec<String>;
    fn get_having(&self) -> Vec<String>;
    fn get_group_by(&mut self) -> Vec<String>;
    fn get_order_by(&mut self) -> Vec<String>;

    fn get_columns(&self) -> Vec<String>;
    fn get_values(&self) -> Vec<String>;

    fn get_sets(&self) -> Vec<String>;
}

impl SQLExplorer {
    pub fn new(sql_provider: Box<dyn SQLProvider>) -> SQLExplorer {
        SQLExplorer {
            sql: SQLStatement::new(sql_provider.get_statement_type()),
            sql_provider,
        }
    }

    pub fn get_sql(&mut self) -> String {
        self.sql.statement_type = self.sql_provider.get_statement_type();
        self.sql.tables = self.sql_provider.get_tables();
        self.sql.select = self.sql_provider.get_select();
        self.sql.join = self.sql_provider.get_join();
        self.sql.inner_join = self.sql_provider.get_inner_join();
        self.sql.outer_join = self.sql_provider.get_outer_join();
        self.sql.left_outer_join = self.sql_provider.get_left_outer_join();
        self.sql.right_outer_join = self.sql_provider.get_right_outer_join();
        self.sql.wheres = self.sql_provider.get_where();
        self.sql.group_by = self.sql_provider.get_group_by();
        self.sql.having = self.sql_provider.get_having();
        self.sql.order_by = self.sql_provider.get_order_by();

        self.sql.columns = self.sql_provider.get_columns();
        self.sql.values = self.sql_provider.get_values();

        self.sql.sets = self.sql_provider.get_sets();

        self.sql.sql()
    }
}

pub fn get_sql(sql_provider: &mut impl SQLProvider) -> String {
    let mut sql = SQLStatement::new(sql_provider.get_statement_type());
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

    sql.sql()
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum StatementType {
    SELECT,
    INSERT,
    UPDATE,
    DELETE,
}

pub struct SQLStatement {
    statement_type: StatementType,
    distinct: bool,
    select: Vec<String>,
    tables: Vec<String>,
    join: Vec<String>,
    inner_join: Vec<String>,
    outer_join: Vec<String>,
    left_outer_join: Vec<String>,
    right_outer_join: Vec<String>,
    wheres: Vec<String>,
    having: Vec<String>,
    group_by: Vec<String>,
    order_by: Vec<String>,
    last_list: Vec<String>,
    // insert
    columns: Vec<String>,
    values: Vec<String>,

    // update
    sets: Vec<String>,
}

impl SQLStatement {
    pub fn new(statement_type: StatementType) -> SQLStatement {
        SQLStatement {
            statement_type,
            sets: Vec::new(),
            select: Vec::new(),
            tables: Vec::new(),
            join: Vec::new(),
            inner_join: Vec::new(),
            outer_join: Vec::new(),
            left_outer_join: Vec::new(),
            right_outer_join: Vec::new(),
            wheres: Vec::new(),
            having: Vec::new(),
            group_by: Vec::new(),
            order_by: Vec::new(),
            last_list: Vec::new(),
            columns: Vec::new(),
            values: Vec::new(),
            distinct: false,
        }
    }

    fn sql_clause(
        &self,
        builder: &mut String,
        keyword: &str,
        parts: &Vec<String>,
        open: &str,
        close: &str,
        conjunction: &str,
    ) {
        if parts.is_empty() {
            return;
        } else {
            if !builder.is_empty() {
                builder.push('\n');
            }
            builder.push_str(&keyword);
            builder.push(' ');
            builder.push_str(&open);
            let mut last = "________";
            for (i, part) in parts.iter().enumerate() {
                if i > 0 && !part.eq(AND) && !part.eq(OR) && !last.eq(AND) && !last.eq(OR) {
                    builder.push_str(&conjunction);
                }
                builder.push_str(part);
                last = part;
            }
            builder.push_str(&close);
        }
    }

    fn build_select(&mut self, mut builder: String) -> String {
        if self.distinct {
            self.sql_clause(&mut builder, "SELECT DISTINCT", &self.select, "", "", ", ")
        } else {
            self.sql_clause(&mut builder, "SELECT", &self.select, "", "", ", ")
        }
        self.sql_clause(&mut builder, "FROM", &self.tables, "", "", ", ");
        self.joins(&mut builder);
        self.sql_clause(&mut builder, "WHERE", &self.wheres, "(", ")", " AND ");
        self.sql_clause(&mut builder, "GROUP BY", &self.group_by, "", "", ", ");
        self.sql_clause(&mut builder, "HAVING", &self.having, "(", ")", " AND ");
        self.sql_clause(&mut builder, "ORDER BY", &self.order_by, "", "", ", ");
        builder
    }

    fn joins(&mut self, builder: &mut String) {
        self.sql_clause(builder, "JOIN", &self.join, "", "", "\nJOIN");
        self.sql_clause(
            builder,
            "INNER JOIN",
            &self.inner_join,
            "",
            "",
            "\nINNER JOIN ",
        );
        self.sql_clause(
            builder,
            "OUTER JOIN",
            &self.outer_join,
            "",
            "",
            "\nOUTER JOIN ",
        );
        self.sql_clause(
            builder,
            "LEFT OUTER JOIN",
            &self.left_outer_join,
            "",
            "",
            "\nLEFT OUTER JOIN ",
        );
        self.sql_clause(
            builder,
            "RIGHT OUTER JOIN",
            &self.right_outer_join,
            "",
            "",
            "\nRIGHT OUTER JOIN ",
        );
    }

    fn build_insert(&mut self, mut builder: String) -> String {
        self.sql_clause(&mut builder, "INSERT INTO", &self.tables, "", "", "");
        self.sql_clause(&mut builder, "", &self.columns, "(", ")", ", ");
        self.sql_clause(&mut builder, "VALUES", &self.values, "(", ")", ", ");
        builder
    }
    fn build_update(&mut self, mut builder: String) -> String {
        self.sql_clause(&mut builder, "UPDATE", &self.tables, "", "", "");
        self.joins(&mut builder);
        self.sql_clause(&mut builder, "SET", &self.sets, "", "", ", ");
        self.sql_clause(&mut builder, "WHERE", &self.wheres, "(", ")", " AND ");
        builder
    }
    fn build_delete(&mut self, mut builder: String) -> String {
        self.sql_clause(&mut builder, "DELETE FROM", &self.tables, "", "", "");
        self.sql_clause(&mut builder, "WHERE", &self.wheres, "(", ")", " AND ");
        builder
    }

    pub fn sql(&mut self) -> String {
        let sql = String::new();
        match self.statement_type {
            StatementType::SELECT => self.build_select(sql),
            StatementType::INSERT => self.build_insert(sql),
            StatementType::UPDATE => self.build_update(sql),
            StatementType::DELETE => self.build_delete(sql),
        }
    }
}

#[cfg(test)]
mod sql_statement_tests {
    use super::*;
    use similar_asserts::assert_eq;

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
}
