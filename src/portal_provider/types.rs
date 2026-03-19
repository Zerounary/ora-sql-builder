use schemars::JsonSchema;
use serde_json::Value;

pub type Id = i64;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Obtainmanner {
    Text,
    Object,
    Ignore,
    Operate,
    SheetNo,
    Triger,
}

impl Default for Obtainmanner {
    fn default() -> Self {
        Obtainmanner::Text
    }
}

impl ToString for Obtainmanner {
    fn to_string(&self) -> String {
        match self {
            Obtainmanner::Text => "text".to_string(),
            Obtainmanner::Object => "object".to_string(),
            Obtainmanner::Ignore => "ignore".to_string(),
            Obtainmanner::Operate => "operate".to_string(),
            Obtainmanner::SheetNo => "sheetNo".to_string(),
            Obtainmanner::Triger => "triger".to_string(),
        }
    }
}

impl Into<String> for Obtainmanner {
    fn into(self) -> String {
        match self {
            Obtainmanner::Text => "text".to_string(),
            Obtainmanner::Object => "object".to_string(),
            Obtainmanner::Ignore => "ignore".to_string(),
            Obtainmanner::Operate => "operate".to_string(),
            Obtainmanner::SheetNo => "sheetNo".to_string(),
            Obtainmanner::Triger => "triger".to_string(),
        }
    }
}

impl From<String> for Obtainmanner {
    fn from(value: String) -> Self {
        Obtainmanner::from(value.as_str())
    }
}

impl From<&str> for Obtainmanner {
    fn from(value: &str) -> Self {
        match value {
            "text" => Obtainmanner::Text,
            "object" => Obtainmanner::Object,
            "ignore" => Obtainmanner::Ignore,
            "operate" => Obtainmanner::Operate,
            "sheetNo" => Obtainmanner::SheetNo,
            "triger" => Obtainmanner::Triger,
            _ => Obtainmanner::Text,
        }
    }
}

#[derive(Default, Clone, JsonSchema)]
pub struct Column {
    pub id: Id,
    pub table_id: Id,
    pub real_table: Option<String>,
    pub current_table: String,
    pub table_name: String,
    pub column_id: Id,
    pub dbname: String,
    pub mask: String,
    pub ref_table_id: Option<Id>,
    pub nullable: bool,
    pub obtainmanner: String,
    pub sequencename: String,
    pub default_value: String,
    pub description: String,
    pub column_name: String,
    pub ref_table: Option<Dk>,
    pub columnlink_tablenames: Vec<String>,
    pub select: Vec<Select>,
    pub value: Option<Value>,
    pub order_by: String,
}

#[derive(Clone, Default, JsonSchema)]
pub struct Dk {
    pub table_id: Id,
    pub table_name: String,
    pub column_id: Id,
    pub dk_column: String,
}

#[derive(Clone, Default, JsonSchema)]
pub struct Select {
    pub code: String,
    pub name: String,
}

#[derive(Default)]
pub struct RefTable {
    pub table_id: Id,
    pub table_name: String,
    pub column_id: Id,
    pub column_name: String,
    pub mask: String,
    pub ref_table_id: Id,
}

#[derive(Default, Clone)]
pub struct PortalProviderOption {
    pub id: Option<Id>,
    pub max_idx: Option<usize>,
    pub is_group: Option<bool>,
    pub table_filter: Option<String>,
}
