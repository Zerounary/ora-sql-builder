pub(crate) const AND: &str = ") \n AND (";
pub(crate) const OR: &str = ") \n OR (";

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StatementType {
    SELECT,
    INSERT,
    UPDATE,
    DELETE,
}

pub struct SQLStatement {
    pub(crate) statement_type: StatementType,
    distinct: bool,
    pub(crate) select: Vec<String>,
    pub(crate) tables: Vec<String>,
    pub(crate) join: Vec<String>,
    pub(crate) inner_join: Vec<String>,
    pub(crate) outer_join: Vec<String>,
    pub(crate) left_outer_join: Vec<String>,
    pub(crate) right_outer_join: Vec<String>,
    pub(crate) wheres: Vec<String>,
    pub(crate) having: Vec<String>,
    pub(crate) group_by: Vec<String>,
    pub(crate) order_by: Vec<String>,
    pub(crate) columns: Vec<String>,
    pub(crate) values: Vec<String>,
    pub(crate) sets: Vec<String>,
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
            columns: Vec::new(),
            values: Vec::new(),
            distinct: false,
        }
    }

    fn sql_clause(
        &self,
        builder: &mut String,
        keyword: &str,
        parts: &[String],
        open: &str,
        close: &str,
        conjunction: &str,
    ) {
        if parts.is_empty() {
            return;
        }

        if !builder.is_empty() {
            builder.push('\n');
        }
        builder.push_str(keyword);
        builder.push(' ');
        builder.push_str(open);
        let mut last = "________";
        for (i, part) in parts.iter().enumerate() {
            if i > 0 && !part.eq(AND) && !part.eq(OR) && !last.eq(AND) && !last.eq(OR) {
                builder.push_str(conjunction);
            }
            builder.push_str(part);
            last = part;
        }
        builder.push_str(close);
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
        self.sql_clause(builder, "INNER JOIN", &self.inner_join, "", "", "\nINNER JOIN ");
        self.sql_clause(builder, "OUTER JOIN", &self.outer_join, "", "", "\nOUTER JOIN ");
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
