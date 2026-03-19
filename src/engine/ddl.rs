use super::dialect::SqlDialect;
use super::query::BuiltQuery;

#[derive(Debug, Clone, PartialEq)]
pub enum DdlValue {
    String(String),
    Number(String),
    Boolean(bool),
    Raw(String),
    Null,
}

impl DdlValue {
    fn render(&self) -> String {
        match self {
            DdlValue::String(value) => format!("'{}'", value.replace('"', "\"").replace("'", "''")),
            DdlValue::Number(value) => value.clone(),
            DdlValue::Boolean(value) => {
                if *value {
                    "TRUE".to_string()
                } else {
                    "FALSE".to_string()
                }
            }
            DdlValue::Raw(value) => value.clone(),
            DdlValue::Null => "NULL".to_string(),
        }
    }
}

impl From<&str> for DdlValue {
    fn from(value: &str) -> Self {
        DdlValue::String(value.to_string())
    }
}

impl From<String> for DdlValue {
    fn from(value: String) -> Self {
        DdlValue::String(value)
    }
}

impl From<bool> for DdlValue {
    fn from(value: bool) -> Self {
        DdlValue::Boolean(value)
    }
}

impl From<i32> for DdlValue {
    fn from(value: i32) -> Self {
        DdlValue::Number(value.to_string())
    }
}

impl From<i64> for DdlValue {
    fn from(value: i64) -> Self {
        DdlValue::Number(value.to_string())
    }
}

impl From<u32> for DdlValue {
    fn from(value: u32) -> Self {
        DdlValue::Number(value.to_string())
    }
}

impl From<u64> for DdlValue {
    fn from(value: u64) -> Self {
        DdlValue::Number(value.to_string())
    }
}

impl From<usize> for DdlValue {
    fn from(value: usize) -> Self {
        DdlValue::Number(value.to_string())
    }
}

impl From<f64> for DdlValue {
    fn from(value: f64) -> Self {
        DdlValue::Number(value.to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDefinition {
    name: String,
    data_type: String,
    nullable: bool,
    default_value: Option<DdlValue>,
    primary_key: bool,
    unique: bool,
}

impl ColumnDefinition {
    pub fn new(name: impl Into<String>, data_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data_type: data_type.into(),
            nullable: true,
            default_value: None,
            primary_key: false,
            unique: false,
        }
    }

    pub fn nullable(mut self) -> Self {
        self.nullable = true;
        self
    }

    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    pub fn default_value(mut self, value: impl Into<DdlValue>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    pub fn default_raw(mut self, sql: impl Into<String>) -> Self {
        self.default_value = Some(DdlValue::Raw(sql.into()));
        self
    }

    pub fn primary_key(mut self) -> Self {
        self.primary_key = true;
        self
    }

    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    fn render(&self) -> String {
        let mut parts = vec![self.name.clone(), self.data_type.clone()];
        if !self.nullable {
            parts.push("NOT NULL".to_string());
        }
        if let Some(default_value) = &self.default_value {
            parts.push(format!("DEFAULT {}", default_value.render()));
        }
        if self.primary_key {
            parts.push("PRIMARY KEY".to_string());
        }
        if self.unique {
            parts.push("UNIQUE".to_string());
        }
        parts.join(" ")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForeignKeyDefinition {
    name: Option<String>,
    columns: Vec<String>,
    referenced_table: String,
    referenced_columns: Vec<String>,
    on_delete: Option<String>,
    on_update: Option<String>,
}

impl ForeignKeyDefinition {
    pub fn new<I, S, J, T>(
        columns: I,
        referenced_table: impl Into<String>,
        referenced_columns: J,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
        J: IntoIterator<Item = T>,
        T: Into<String>,
    {
        Self {
            name: None,
            columns: collect_identifiers(columns),
            referenced_table: referenced_table.into(),
            referenced_columns: collect_identifiers(referenced_columns),
            on_delete: None,
            on_update: None,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn on_delete(mut self, action: impl Into<String>) -> Self {
        self.on_delete = Some(action.into());
        self
    }

    pub fn on_update(mut self, action: impl Into<String>) -> Self {
        self.on_update = Some(action.into());
        self
    }

    fn render(&self) -> String {
        let mut sql = String::new();
        if let Some(name) = &self.name {
            sql.push_str(&format!("CONSTRAINT {} ", name));
        }
        sql.push_str(&format!(
            "FOREIGN KEY ({}) REFERENCES {} ({})",
            self.columns.join(", "),
            self.referenced_table,
            self.referenced_columns.join(", ")
        ));
        if let Some(action) = &self.on_delete {
            sql.push_str(&format!(" ON DELETE {}", action));
        }
        if let Some(action) = &self.on_update {
            sql.push_str(&format!(" ON UPDATE {}", action));
        }
        sql
    }
}

pub struct CreateTableBuilder {
    table: String,
    if_not_exists: bool,
    columns: Vec<ColumnDefinition>,
    primary_keys: Vec<String>,
    unique_keys: Vec<Vec<String>>,
    foreign_keys: Vec<ForeignKeyDefinition>,
}

impl CreateTableBuilder {
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            if_not_exists: false,
            columns: Vec::new(),
            primary_keys: Vec::new(),
            unique_keys: Vec::new(),
            foreign_keys: Vec::new(),
        }
    }

    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self
    }

    pub fn column(mut self, column: ColumnDefinition) -> Self {
        self.columns.push(column);
        self
    }

    pub fn primary_key<I, S>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.primary_keys = collect_identifiers(columns);
        self
    }

    pub fn unique<I, S>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.unique_keys.push(collect_identifiers(columns));
        self
    }

    pub fn foreign_key(mut self, foreign_key: ForeignKeyDefinition) -> Self {
        self.foreign_keys.push(foreign_key);
        self
    }

    pub fn build(self, _dialect: &dyn SqlDialect) -> BuiltQuery {
        let mut definitions = self
            .columns
            .into_iter()
            .map(|column| column.render())
            .collect::<Vec<_>>();
        if !self.primary_keys.is_empty() {
            definitions.push(format!("PRIMARY KEY ({})", self.primary_keys.join(", ")));
        }
        for unique_key in self.unique_keys {
            definitions.push(format!("UNIQUE ({})", unique_key.join(", ")));
        }
        for foreign_key in self.foreign_keys {
            definitions.push(foreign_key.render());
        }

        BuiltQuery {
            sql: format!(
                "CREATE TABLE {}{} ({})",
                if self.if_not_exists {
                    "IF NOT EXISTS "
                } else {
                    ""
                },
                self.table,
                definitions.join(", ")
            ),
            params: Vec::new(),
        }
    }
}

enum AlterTableOperation {
    AddColumn(ColumnDefinition),
    DropColumn(String),
    RenameColumn { from: String, to: String },
    RenameTable(String),
    AddConstraint(String),
    Raw(String),
}

impl AlterTableOperation {
    fn render(self) -> String {
        match self {
            AlterTableOperation::AddColumn(column) => format!("ADD COLUMN {}", column.render()),
            AlterTableOperation::DropColumn(column) => format!("DROP COLUMN {}", column),
            AlterTableOperation::RenameColumn { from, to } => {
                format!("RENAME COLUMN {} TO {}", from, to)
            }
            AlterTableOperation::RenameTable(name) => format!("RENAME TO {}", name),
            AlterTableOperation::AddConstraint(sql) => sql,
            AlterTableOperation::Raw(sql) => sql,
        }
    }
}

pub struct AlterTableBuilder {
    table: String,
    operations: Vec<AlterTableOperation>,
}

impl AlterTableBuilder {
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            operations: Vec::new(),
        }
    }

    pub fn add_column(mut self, column: ColumnDefinition) -> Self {
        self.operations.push(AlterTableOperation::AddColumn(column));
        self
    }

    pub fn drop_column(mut self, column: impl Into<String>) -> Self {
        self.operations
            .push(AlterTableOperation::DropColumn(column.into()));
        self
    }

    pub fn rename_column(
        mut self,
        from: impl Into<String>,
        to: impl Into<String>,
    ) -> Self {
        self.operations.push(AlterTableOperation::RenameColumn {
            from: from.into(),
            to: to.into(),
        });
        self
    }

    pub fn rename_table(mut self, name: impl Into<String>) -> Self {
        self.operations
            .push(AlterTableOperation::RenameTable(name.into()));
        self
    }

    pub fn add_constraint(mut self, sql: impl Into<String>) -> Self {
        self.operations
            .push(AlterTableOperation::AddConstraint(sql.into()));
        self
    }

    pub fn raw(mut self, sql: impl Into<String>) -> Self {
        self.operations.push(AlterTableOperation::Raw(sql.into()));
        self
    }

    pub fn build(self, _dialect: &dyn SqlDialect) -> BuiltQuery {
        let sql = self
            .operations
            .into_iter()
            .map(|operation| format!("ALTER TABLE {} {}", self.table, operation.render()))
            .collect::<Vec<_>>()
            .join("; ");
        BuiltQuery {
            sql,
            params: Vec::new(),
        }
    }
}

pub struct DropTableBuilder {
    table: String,
    if_exists: bool,
    cascade: bool,
}

impl DropTableBuilder {
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            if_exists: false,
            cascade: false,
        }
    }

    pub fn if_exists(mut self) -> Self {
        self.if_exists = true;
        self
    }

    pub fn cascade(mut self) -> Self {
        self.cascade = true;
        self
    }

    pub fn build(self, _dialect: &dyn SqlDialect) -> BuiltQuery {
        BuiltQuery {
            sql: format!(
                "DROP TABLE {}{}{}",
                if self.if_exists { "IF EXISTS " } else { "" },
                self.table,
                if self.cascade { " CASCADE" } else { "" }
            ),
            params: Vec::new(),
        }
    }
}

fn collect_identifiers<I, S>(items: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    items.into_iter().map(Into::into).collect()
}
