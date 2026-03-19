use schemars::JsonSchema;
use serde_json::Value;
use std::collections::HashMap;
use wildmatch::WildMatch;

use crate::sql::{SQLProvider, StatementType};

fn sanitize_filter_value(value: &str) -> String {
    let mut sanitized = String::new();
    let chars: Vec<char> = value.chars().collect();
    let mut index = 0;

    while index < chars.len() {
        if chars[index] == '%'
            && index + 2 < chars.len()
            && chars[index + 1].is_ascii_hexdigit()
            && chars[index + 2].is_ascii_hexdigit()
        {
            index += 3;
            continue;
        }

        sanitized.push(chars[index]);
        index += 1;
    }

    sanitized
}

pub type Id = i64;
#[derive(Clone)]
pub struct PortalProvider {
    statement_type: StatementType,
    user_id: Id,
    data: Vec<Column>,
    args: PortalProviderOption,
    ref_table_column_map: HashMap<String, (String, String)>,
}

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
    pub id: Option<Id>,               // 修改，查询
    pub max_idx: Option<usize>,       // 新增、修改、查询
    pub is_group: Option<bool>,       // 查询
    pub table_filter: Option<String>, // 修改，查询
}

impl PortalProvider {
    pub fn new(user_id: Id, data: Vec<Column>) -> Self {
        PortalProvider {
            statement_type: StatementType::SELECT,
            user_id,
            data,
            args: PortalProviderOption::default(),
            ref_table_column_map: HashMap::new(),
        }
    }

    pub fn new_opt(user_id: Id, data: Vec<Column>, args: PortalProviderOption) -> Self {
        PortalProvider {
            statement_type: StatementType::SELECT,
            user_id,
            data,
            args,
            ref_table_column_map: HashMap::new(),
        }
    }
    pub fn statement_type(&mut self, statement_type: StatementType) {
        self.statement_type = statement_type;
    }

    fn table_with_alias(real_table: Option<&String>, current_table: &str) -> String {
        match real_table {
            Some(real_table) => format!("{real_table} {current_table}"),
            None => current_table.to_string(),
        }
    }

    fn is_reference_lookup_column(column: &Column) -> bool {
        column.ref_table_id.is_some()
            && column.ref_table.is_some()
            && matches!(
                Obtainmanner::from(column.obtainmanner.as_str()),
                Obtainmanner::Object | Obtainmanner::Operate
            )
    }

    fn is_sheet_no_column(column: &Column) -> bool {
        Obtainmanner::from(column.obtainmanner.as_str()) == Obtainmanner::SheetNo
    }

    fn matches_mask(&self, column: &Column) -> bool {
        column.mask.chars().nth(self.args.max_idx.unwrap_or_default()) == Some('1')
    }

    fn select_expressions(&self, main_table: &str, column: &Column) -> Vec<String> {
        if Self::is_reference_lookup_column(column) {
            let dk = column.ref_table.as_ref().unwrap();
            return vec![
                format!(
                    "{main_table}.{dbname} as \"{dbname}\"",
                    dbname = column.dbname,
                ),
                format!(
                    "(select {dk_column} as dk from {table_name} x where id = {main_table}.{dbname}) as \"{dbname}.dk\"",
                    dk_column = dk.dk_column,
                    table_name = dk.table_name,
                    dbname = column.dbname,
                ),
            ];
        }

        if column.dbname.contains(";") {
            let fk_column = column.dbname.split(';').last().unwrap();
            return vec![format!(
                "{table_alias}.{dbname} as \"{column_id}\"",
                table_alias = self.get_column_link_alias(column.dbname.clone()),
                dbname = fk_column,
                column_id = column.column_id
            )];
        }

        if WildMatch::new("*(*)").matches(&column.dbname) {
            return vec![format!(
                "{dbname} as \"{column_id}\"",
                dbname = column.dbname,
                column_id = column.column_id
            )];
        }

        if column.dbname.contains('.') {
            return vec![format!(
                "{dbname} as \"{dbname}\"",
                dbname = column.dbname
            )];
        }

        vec![format!(
            "{main_table}.{dbname} as \"{dbname}\"",
            dbname = column.dbname,
        )]
    }

    fn filter_target(&self, filter: &Column) -> (String, String) {
        if filter.dbname.contains(';') {
            (
                self.get_column_link_alias(filter.dbname.to_string()),
                filter
                    .dbname
                    .split(';')
                    .collect::<Vec<&str>>()
                    .last()
                    .unwrap()
                    .to_string(),
            )
        } else {
            (filter.current_table.to_string(), filter.dbname.to_string())
        }
    }

    fn string_filter_clauses(&self, table_name: &str, column_name: &str, value: &str) -> Vec<String> {
        let rs_condiction = sanitize_filter_value(value).replace("'", "''");
        if rs_condiction.starts_with('=') {
            return vec![format!(
                "{}.{} = '{}'",
                table_name,
                column_name,
                rs_condiction.trim_start_matches('=')
            )];
        }

        if rs_condiction.contains(' ') {
            let subs: Vec<String> = rs_condiction
                .split_whitespace()
                .map(|sub| {
                    format!(
                        "{}.{} like '%{}%'",
                        table_name,
                        column_name,
                        sub.trim()
                    )
                })
                .collect();
            return vec![format!("({})", subs.join(" or "))];
        }

        vec![format!(
            "{}.{} = '{}'",
            table_name, column_name, rs_condiction
        )]
    }

    fn array_filter_clause(&self, table_name: &str, column_name: &str, array: &[Value]) -> String {
        let mut limit: Vec<String> = Vec::new();
        for obj in array {
            if obj.is_number() {
                limit.push(obj.to_string());
            } else if obj.is_string() {
                let s = obj.as_str().unwrap().to_string().replace("'", "''");
                limit.push(format!("'{}'", s));
            }
        }

        format!("{}.{} IN ({})", table_name, column_name, limit.join(","))
    }

    fn object_filter_clauses(&self, table_name: &str, column_name: &str, value: &serde_json::Map<String, Value>) -> Vec<String> {
        let mut list = Vec::new();
        let ty = value.get("type");
        if Some(&Value::String("between".to_string())) == ty {
            let begin = value.get("begin");
            let end = value.get("end");
            if let Some(Value::Number(begin)) = begin {
                list.push(format!("{}.{} >= {}", table_name, column_name, begin));
            }
            if let Some(Value::Number(end)) = end {
                list.push(format!("{}.{} <= {}", table_name, column_name, end));
            }
        }
        list
    }

    fn insert_init_columns() -> Vec<String> {
        [
            "id",
            "ad_client_id",
            "ad_org_id",
            "ownerid",
            "modifiered",
            "creationdate",
            "modifieddate",
        ]
        .into_iter()
        .map(str::to_string)
        .collect()
    }

    fn insert_init_values(&self) -> Vec<String> {
        vec![
            self.args.id.unwrap().to_string(),
            "37".to_string(),
            "27".to_string(),
            self.user_id.to_string(),
            self.user_id.to_string(),
            "sysdate".to_string(),
            "sysdate".to_string(),
        ]
    }

    fn insert_value_expression(&self, column: &Column, condition: Value) -> Option<String> {
        match condition {
            Value::Null => Some("null".to_string()),
            Value::Bool(_) => todo!(),
            Value::Number(num) => Some(num.to_string()),
            Value::String(s) => {
                if s.is_empty() {
                    Some("null".to_string())
                } else if !s.starts_with('0') && s.chars().all(|c| c.is_ascii_digit()) {
                    Some(s)
                } else if Self::is_reference_lookup_column(column) {
                    let dk = column.ref_table.as_ref().unwrap();
                    Some(format!(
                        "(select id from {} where {} = '{}')",
                        dk.table_name,
                        dk.dk_column,
                        s
                    ))
                } else {
                    Some(format!("'{}'", s.replace("'", "''")))
                }
            }
            _ => None,
        }
    }

    fn update_set_clause(&self, column: &Column, value: &Value) -> Option<String> {
        match value {
            Value::Number(num) => Some(format!("{} = {}", column.dbname, num)),
            Value::String(s) => Some(format!("{} = '{}'", column.dbname, s.replace("'", "''"))),
            _ => None,
        }
    }

    fn get_main_table(&mut self) -> String {
        let tables = self.get_tables();
        let mut main_table = tables.get(0).unwrap().to_string();
        for table in tables {
            if !table.contains(" ") {
                main_table = table.to_string();
                break;
            }
        }
        if main_table.contains(" ") {
            main_table = main_table.split_whitespace().last().unwrap().to_string();
        }
        main_table
    }

    fn get_column_link_alias(&self, column_link: String) -> String {
        let last = column_link.chars().filter(|c| c.eq(&';')).count();
        return self.get_column_link_table(column_link, last);
    }

    fn get_column_link_table(&self, column_link: String, no: usize) -> String {
        let idx = ordinal_index_of(&column_link, ';', no);
        let link_table = column_link.get(0..idx).unwrap();
        self.ref_table_column_map
            .get(link_table)
            .unwrap()
            .0.to_string()
    }

    fn is_insert_column(&self, column: &Column) -> bool {
        // 是否是默认的后台插入字段，界面上可能没有
        let is_back_insert_column = !column.nullable
            && !column.dbname.to_uppercase().eq("ID")
            && column.obtainmanner != Obtainmanner::Triger.to_string();
        let is_not_calc_column = !column.dbname.contains(' ');
        (self.matches_mask(column) && is_not_calc_column)
            || (is_back_insert_column && is_not_calc_column)
    }
}

impl SQLProvider for PortalProvider {
    fn get_statement_type(&self) -> crate::sql::StatementType {
        self.statement_type
    }

    fn get_select(&mut self) -> Vec<String> {
        let mut list = Vec::new();
        if self.get_statement_type() == StatementType::SELECT {
            let main_table: String = self.get_main_table();
            match self.args.is_group {
                Some(false) | None => {
                    list.push(format!("{}.id as \"id\"", main_table));
                }
                _ => {}
            }
            if self.args.max_idx.is_some() {
                for column in &self.data {
                    if self.matches_mask(column) {
                        list.extend(self.select_expressions(&main_table, column));
                    }
                }
            }
        }
        list
    }

    /// 表和关联表
    /// column_name   字段名称
    /// table_name    表名
    /// current_table 字段配置使用的表名
    /// real_table    真实数据库表名(可能不存在)
    /// ref_tables    字段为Column_link时的字段关联表
    fn get_tables(&mut self) -> Vec<String> {
        let mut tablenames: Vec<String> = Vec::new();
        let mut ref_table_save: Vec<String> = Vec::new();
        let mut count = 0;
        self.ref_table_column_map.clear();

        // 直接添加主表
        if self.get_statement_type() == StatementType::SELECT {
            if !self.data.is_empty() {
                let first_column = self.data.first().unwrap();
                push_unique(
                    &mut tablenames,
                    Self::table_with_alias(first_column.real_table.as_ref(), &first_column.current_table),
                );
            }
        }

        // 添加关联表
        for column in &self.data {
            match self.get_statement_type() {
                StatementType::SELECT => {
                    let ref_table_names = &column.columnlink_tablenames;
                    // 获取数据库实际表名
                    let table = Self::table_with_alias(column.real_table.as_ref(), &column.current_table);

                    if ref_table_names.is_empty() {
                        // 是否是需要获取DK的外键
                        // if column.ref_table.is_some() {
                        //     let dk = column.ref_table.clone().unwrap();
                        //     count += 1;
                        //     let alias_name: String = format!("b{}", count);

                        //     self.dk_column_map
                        //         .insert(column.id, alias_name.to_string());
                        //     push_unique(
                        //         &mut tablenames,
                        //         format!("{} {}", dk.table_name, alias_name),
                        //     );
                        // } else {
                        //     push_unique(&mut tablenames, table);
                        // }
                        push_unique(&mut tablenames, table);
                    } else {
                        // A;B;C 字段，查出 A,B 作为 refTableNames
                        for (i, ref_table_name) in ref_table_names.iter().enumerate() {
                            // A, B
                            // i = 0 A
                            // i = 1 A;B
                            let end_idx = ordinal_index_of(&column.dbname, ';', i + 1);
                            let sub_ref_column_name = &column.dbname[0..end_idx];
                            // 已经记录过了扩展根本表不在添加
                            if ref_table_save.contains(&sub_ref_column_name.to_string()) {
                                continue;
                            }
                            count += 1;
                            let alias_name: String = format!("a{}", count);

                            // 需要记录到refTableColumnMap， 以供支持同一个表多个字段的联表情况
                            // A;B;C => A ,  A;B 根据i变化， 缓存每个 columnLink 对应的关联表名称

                            self.ref_table_column_map.insert(
                                sub_ref_column_name.to_string(),
                                (alias_name.to_string(), ref_table_name.to_string()),
                            );
                            ref_table_save.push(sub_ref_column_name.to_string());
                            // push_unique(
                            //     &mut tablenames,
                            //     format!("{} {}", ref_table_name, alias_name),
                            // );
                        }
                    }
                }
                StatementType::UPDATE | StatementType::DELETE => {
                    push_unique(
                        &mut tablenames,
                        Self::table_with_alias(column.real_table.as_ref(), &column.current_table),
                    );
                    break;
                }
                StatementType::INSERT => {
                    // INSERT INTO {表名} 没有别名
                    let real_table = column.real_table.clone();
                    let current_table = column.current_table.clone();
                    let table = real_table.unwrap_or(current_table);
                    push_unique(&mut tablenames, table);
                    break;
                }
            }
        }
        tablenames
    }

    fn get_join(&self) -> Vec<String> {
        Vec::new()
    }

    fn get_inner_join(&self) -> Vec<String> {
        Vec::new()
    }

    fn get_outer_join(&self) -> Vec<String> {
        Vec::new()
    }

    fn get_left_outer_join(&mut self) -> Vec<String> {
        let main_table = self.get_main_table();
        let mut ref_table_save: Vec<String> = Vec::new();
        let mut join_statements: Vec<String> = Vec::new();
        for column in &self.data {
            // 处理 外键 dk 的联表语句
            // if column.ref_table.is_some() {
            //     let right_table = self.dk_column_map.get(&column.id).unwrap();
            //     push_unique(
            //         &mut join_statements,
            //         format!("{}.{} = {}.id(+)", main_table, column.dbname, right_table),
            //     )
            // }

            // 处理 columnlink 的联表语句
            if column.dbname.contains(";") {
                let mut columns: Vec<&str> = column.dbname.split(";").collect(); // C_STORE_ID;C_STOREKIND_ID;NAME ==> [C_STORE_ID, C_STOREKIND_ID, NAME]
                columns.remove(columns.len() - 1); // 移除最后的字段名
                for (i, fk_column) in columns.iter().enumerate() {
                    // if columns.len
                    let end_idx = ordinal_index_of(&column.dbname, ';', i + 1);
                    let column_link = &column.dbname[0..end_idx]; // C_STORE_ID;C_STOREKIND_ID
                    let (alias, right_table) = self.ref_table_column_map.get(column_link).unwrap(); // C_STORE_ID;C_STOREKIND_ID ==> a11
                                                                                                    // 如果存在 A;B;C 和 A;C;D 同一个字段A的扩展至关联一回
                                                                                                    // 如果存在 B;A;C 和 C;A;B 后面的表要关联
                    let alias_name = if i == 0 {
                        if ref_table_save.contains(&column_link.to_string()) {
                            continue;
                        }
                        ref_table_save.push(column_link.to_string());

                        alias.to_string()
                    } else {
                        self.get_column_link_alias(column_link.to_string())
                    };
                    let left_table = if i == 0 {
                        main_table.to_string()
                    } else {
                        alias_name
                    };
                    push_unique(
                        &mut join_statements,
                        format!(
                            "{join_table} {alias} ON {left_table}.{dbname} = {alias}.id",
                            join_table = right_table,
                            alias = alias,
                            dbname = fk_column,
                            left_table = left_table
                        ),
                    )
                }
            }
        }
        // 追加join语句
        join_statements
    }

    fn get_right_outer_join(&self) -> Vec<String> {
        Vec::new()
    }

    /// 过滤条件
    fn get_where(&mut self) -> Vec<String> {
        let mut list = Vec::new();
        match self.get_statement_type() {
            StatementType::SELECT => {
                if !self.get_tables().is_empty() {
                    let main_table = self.get_main_table();

                    // 权限过滤条件
                    if let Some(table_filter) = &self.args.table_filter {
                        if !table_filter.is_empty() {
                            list.push(table_filter.to_string());
                        }
                    }

                    // 过滤条件
                    let filter_columns: Vec<&Column> =
                        self.data.iter().filter(|e| e.value.is_some()).collect();

                    for filter in filter_columns {
                        let (table_name, column_name) = self.filter_target(filter);

                        // 没值的不过滤
                        if filter.value.is_none() {
                            continue;
                        }

                        // 处理不同类型的值作为过滤条件
                        match filter.value.as_ref().unwrap() {
                            Value::Null => continue,
                            Value::Bool(b) => {
                                if *b {
                                    list.push(format!("{}.{} = 'Y'", table_name, column_name));
                                } else {
                                    list.push(format!("{}.{} = 'N'", table_name, column_name));
                                }
                            }
                            Value::Number(num) => {
                                list.push(format!("{}.{} = {}", table_name, column_name, num));
                            }
                            Value::String(s) => {
                                if filter.obtainmanner == Obtainmanner::Object.to_string()
                                    && s.starts_with("<")
                                    && s.ends_with(">")
                                {
                                    // 多选XML
                                    unimplemented!()
                                } else {
                                    // 普通字符串
                                    list.extend(self.string_filter_clauses(&table_name, &column_name, s));
                                }
                            }
                            Value::Array(array) => {
                                list.push(self.array_filter_clause(&table_name, &column_name, array))
                            }
                            Value::Object(obj) => {
                                list.extend(self.object_filter_clauses(&table_name, &column_name, obj));
                            }
                        }
                    }

                    // 主表id
                    if let Some(id) = &self.args.id {
                        list.push(format!("{}.id = {}", main_table, id));
                    }
                }
            }
            StatementType::UPDATE => {
                if self.args.table_filter.is_some() {
                    list.push(self.args.table_filter.clone().unwrap());
                }
                list.push(format!("id = {}", self.args.id.unwrap()))
            }
            StatementType::INSERT | StatementType::DELETE => {}
        }
        list
    }

    fn get_having(&self) -> Vec<String> {
        Vec::new()
    }

    // 分组条件
    fn get_group_by(&mut self) -> Vec<String> {
        let mut list = Vec::new();
        if self.get_statement_type() == StatementType::SELECT {
            if self.args.is_group == Some(true) {
                let select = self.get_select();
                for column in &select {
                    // 跳过 函数统计 SUM(qty) ，跳过 子查询(select xx)
                    if !WildMatch::new("*(*)*").matches(column) {
                        // 跳过函数统计字段
                        if let Some((select_column, _)) = column.split_once(" as ") {
                            list.push(select_column.to_string());
                        }
                    }
                }
            }
        }
        list
    }

    // 排序条件
    fn get_order_by(&mut self) -> Vec<String> {
        let mut list = Vec::new();
        if self.get_statement_type() == StatementType::SELECT {
            if self.args.is_group == Some(true) {
                let mut group_by_list = self.get_group_by();
                list.append(&mut group_by_list);
            } else {
                let columns: Vec<&Column> = self
                    .data
                    .iter()
                    .filter(|e| !e.order_by.is_empty())
                    .collect();

                for column in columns {
                    match column.order_by.as_str() {
                        "+" => {
                            list.push(format!(
                                "{}.{} asc nulls first",
                                column.current_table, column.dbname
                            ));
                        }
                        "-" => {
                            list.push(format!(
                                "{}.{} desc nulls first",
                                column.current_table, column.dbname
                            ));
                        }
                        _ => {}
                    }
                }
            }
        }
        list
    }

    fn get_columns(&self) -> Vec<String> {
        let mut list = Vec::new();
        if self.get_statement_type() == StatementType::INSERT {
            list.extend(Self::insert_init_columns());

            for column in self.data.iter().filter(|c| self.is_insert_column(c)) {
                // 表单使用序号生成器
                if Self::is_sheet_no_column(column) && !column.sequencename.is_empty() {
                    list.push(column.dbname.to_string());
                    continue;
                }
                list.push(column.dbname.to_string());
            }
        }
        list
    }

    fn get_values(&self) -> Vec<String> {
        let mut list = Vec::new();
        if self.get_statement_type() == StatementType::INSERT {
            list.extend(self.insert_init_values());
            for column in self.data.iter().filter(|c| self.is_insert_column(c)) {
                if Self::is_sheet_no_column(column) {
                    list.push(format!("get_sequenceno('{}', 37)", column.sequencename));
                    continue;
                }
                let condition = column
                    .value
                    .clone()
                    .unwrap_or(Value::String(column.default_value.to_string()));

                if let Some(value) = self.insert_value_expression(column, condition) {
                    list.push(value);
                }
            }
        }
        list
    }

    fn get_sets(&self) -> Vec<String> {
        let mut list = Vec::new();
        list.push(format!("modifierid={}", self.user_id));
        list.push("modifieddate = sysdate".to_string());
        if self.get_statement_type() == StatementType::UPDATE {
            for column in &self.data {
                if self.matches_mask(column) {
                    if let Some(value) = &column.value {
                        if let Some(set_clause) = self.update_set_clause(column, value) {
                            list.push(set_clause);
                        }
                    }
                }
            }
        }
        list
    }
}

/// 获取指定下标在字符{arg} 在 {dbname} 中，第 {x} 次出现的位置
/// ordinal_index_of("a;b;c", ';', 2) => 3
fn ordinal_index_of(dbname: &str, arg: char, x: usize) -> usize {
    let mut index = 0;
    let mut count = 0;
    for (i, ch) in dbname.char_indices() {
        if ch == arg {
            count += 1;
            if count == x {
                index = i;
                break;
            }
        }
    }
    index
}

/// push_unique
fn push_unique(list: &mut Vec<String>, item: String) {
    if !list.contains(&item) {
        list.push(item.to_string());
    }
}

#[cfg(test)]
mod sanitize_filter_value_tests {
    use super::sanitize_filter_value;
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

#[cfg(test)]
mod portal_provider_from {
    use std::time::Instant;

    use super::*;
    use crate::sql::SQLExplorer;
    use serde_json::json;
    use similar_asserts::assert_eq;
    use wildmatch::WildMatch;

    static USER_ID: &'static str = "893";

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
            // empty_select(); 0.008ms
            // single_select(); // 4ms == use wildmatch ==> 0.01ms
            virtual_table_select(); // 4ms == use wildmatch ==> 0.01ms
            // column_link_select(); // 0.04ms
        }
        let end = now.elapsed();
        println!("{}次执行{}ms", count, end.as_millis());
        // println!("1ms/1次", count / end.as_millis());
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
            // 字符串
            Column {
                dbname: "name".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                value: Some(Value::String("名称".to_string())),
                ..Column::default()
            },
            // 整数
            Column {
                dbname: "qty".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                value: Some(json!(30)),
                ..Column::default()
            },
            // 小数
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
            // 字符串
            Column {
                dbname: "code".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                default_value: "默认值".to_string(),
                ..Column::default()
            },
            // 字符串
            Column {
                dbname: "name".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                value: Some(Value::String("名称".to_string())),
                ..Column::default()
            },
            // 整数
            Column {
                dbname: "qty".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                value: Some(json!(30)),
                ..Column::default()
            },
            // 小数
            Column {
                dbname: "amt".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                value: Some(json!(30.13)),
                ..Column::default()
            },
            // 单据
            Column {
                dbname: "docno".to_string(),
                mask: "1".to_string(),
                current_table: "m_retail".to_string(),
                value: Some(json!("")),
                obtainmanner: Obtainmanner::SheetNo.to_string(),
                sequencename: "RE".to_string(),
                ..Column::default()
            },
            // 外键 数字id 录入
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
            // 外键 字符串id 录入
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
            // 外键 DK 录入
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
            // 过滤计算字段
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
        // let now = Instant::now();
        // for _i in 0..count {
        //     if Regex::new(r"\w+\(.*\)").unwrap().is_match(dbname) {

        //     }
        // }

        // println!("{}", now.elapsed().as_millis());
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
