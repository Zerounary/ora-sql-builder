use serde_json::Value;
use std::collections::HashMap;
use wildmatch::WildMatch;

use crate::sql::{SQLProvider, StatementType};
mod helpers;
mod types;

use helpers::{ordinal_index_of, push_unique, sanitize_filter_value};
pub use types::{Column, Dk, Id, Obtainmanner, PortalProviderOption, RefTable, Select};

#[derive(Clone)]
pub struct PortalProvider {
    statement_type: StatementType,
    user_id: Id,
    data: Vec<Column>,
    args: PortalProviderOption,
    ref_table_column_map: HashMap<String, (String, String)>,
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

#[cfg(test)]
mod tests;