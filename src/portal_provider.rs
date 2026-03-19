use schemars::{schema_for, JsonSchema};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use wildmatch::WildMatch;

use crate::sql::{SQLProvider, StatementType};

pub type Id = i64;
#[derive(Clone)]
pub struct PortalProvider {
    statement_type: StatementType,
    user_id: Id,
    data: Vec<Column>,
    args: PortalProviderOption,
    dk_column_map: HashMap<Id, String>,
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
        match value.as_str() {
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
            dk_column_map: HashMap::new(),
            ref_table_column_map: HashMap::new(),
        }
    }

    pub fn new_opt(user_id: Id, data: Vec<Column>, args: PortalProviderOption) -> Self {
        PortalProvider {
            statement_type: StatementType::SELECT,
            user_id,
            data,
            args,
            dk_column_map: HashMap::new(),
            ref_table_column_map: HashMap::new(),
        }
    }
    pub fn statement_type(&mut self, statement_type: StatementType) {
        self.statement_type = statement_type;
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
        (column
            .mask
            .chars()
            .nth(self.args.max_idx.unwrap_or_default())
            .eq(&Some('1'))
            && is_not_calc_column)
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
            if let PortalProviderOption { max_idx, .. } = &self.args {
                for column in &self.data {
                    if let Some(idx) = max_idx {
                        if column.mask.chars().nth(*idx) == Some('1') {
                            if column.ref_table_id.is_some()
                                && column.ref_table.is_some()
                                && (vec![Obtainmanner::Object, Obtainmanner::Operate]
                                    .contains(&column.obtainmanner.clone().into()))
                            {
                                // 类似 C_STORE_ID 这样的外键表
                                // 查询主表
                                list.push(format!(
                                    "{main_table}.{dbname} as \"{dbname}\"",
                                    main_table = main_table,
                                    dbname = column.dbname,
                                ));
                                // 查询dk
                                let dk = column.ref_table.clone().unwrap();
                                list.push(format!(
                                    "(select {dk_column} as dk from {table_name} x where id = {main_table}.{dbname}) as \"{dbname}.dk\""
                                    ,
                                    dk_column = dk.dk_column,
                                    table_name = dk.table_name,
                                    main_table = main_table,
                                    dbname = column.dbname
                                ));
                                // 联表方式查询
                                // let dk = column.ref_table.clone().unwrap();
                                // let dk_alias = self.dk_column_map.get(&column.id).unwrap();
                                // list.push(format!(
                                //     "{dk_alias}.{dk_column} as \"{dbname}.dk\"",
                                //     dk_alias = dk_alias,
                                //     dk_column = dk.dk_column,
                                //     dbname = column.dbname
                                // ));
                            } else {
                                if column.dbname.contains(";") {
                                    // column link  A;B;C 这种
                                    let fk_column = column.dbname.split(";").last().unwrap();
                                    list.push(format!(
                                        "{table_alias}.{dbname} as \"{column_id}\"",
                                        table_alias =
                                            self.get_column_link_alias(column.dbname.clone()),
                                        dbname = fk_column,
                                        column_id = column.column_id
                                    ));
                                } else if WildMatch::new("*(*)").matches(&column.dbname) {
                                    list.push(format!(
                                        "{dbname} as \"{column_id}\"",
                                        dbname = column.dbname,
                                        column_id = column.column_id
                                    ));
                                } else if column.dbname.contains(".") {
                                    // (c_country.id + 111)
                                    list.push(format!(
                                        "{dbname} as \"{dbname}\"",
                                        dbname = column.dbname
                                    ))
                                } else {
                                    list.push(format!(
                                        "{main_table}.{dbname} as \"{dbname}\"",
                                        main_table = main_table,
                                        dbname = column.dbname,
                                    ))
                                }
                            }
                        }
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
        let mut list = Vec::new();
        let mut tablenames: Vec<String> = Vec::new();
        let mut ref_table_save: Vec<String> = Vec::new();
        let mut count = 0;

        // 直接添加主表
        if self.get_statement_type() == StatementType::SELECT {
            if !self.data.is_empty() {
                let first_column = self.data.first().unwrap();
                if let Some(real_table) = &first_column.real_table {
                    push_unique(
                        &mut tablenames,
                        format!(
                            "{table} {table_alias}",
                            table = real_table,
                            table_alias = first_column.current_table
                        ),
                    );
                } else {
                    push_unique(&mut tablenames, first_column.current_table.clone());
                }
            }
        }

        // 添加关联表
        for column in &self.data {
            match self.get_statement_type() {
                StatementType::SELECT => {
                    let ref_table_names = &column.columnlink_tablenames;
                    // 获取数据库实际表名
                    let table = if let Some(real_table) = &column.real_table {
                        format!(
                            "{table} {table_alias}",
                            table = real_table,
                            table_alias = column.current_table
                        )
                    } else {
                        column.current_table.clone()
                    };

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
                    if let Some(real_table) = &column.real_table {
                        push_unique(
                            &mut tablenames,
                            format!(
                                "{table} {table_alias}",
                                table = real_table,
                                table_alias = column.current_table
                            ),
                        );
                    } else {
                        push_unique(&mut tablenames, column.current_table.clone());
                    }
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
        for tablename in tablenames {
            list.push(tablename);
        }
        list
    }

    fn get_join(&self) -> Vec<String> {
        let mut list = Vec::new();
        list
    }

    fn get_inner_join(&self) -> Vec<String> {
        let mut list = Vec::new();
        list
    }

    fn get_outer_join(&self) -> Vec<String> {
        let mut list = Vec::new();
        list
    }

    fn get_left_outer_join(&mut self) -> Vec<String> {
        let mut list = Vec::new();
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
        for join_statement in join_statements {
            list.push(join_statement);
        }
        list
    }

    fn get_right_outer_join(&self) -> Vec<String> {
        let mut list = Vec::new();
        list
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
                        let (table_name, column_name) = if filter.dbname.contains(";") {
                            let table_name = self.get_column_link_alias(filter.dbname.to_string());
                            let column_name = filter
                                .dbname
                                .split(";")
                                .collect::<Vec<&str>>()
                                .last()
                                .unwrap()
                                .to_string();
                            (table_name, column_name)
                        } else {
                            (filter.current_table.to_string(), filter.dbname.to_string())
                        };

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
                                    let rs_condiction = s.to_string().replace("'", "''");
                                    if rs_condiction.starts_with("=") {
                                        list.push(format!(
                                            "{}.{} = '{}'",
                                            table_name,
                                            column_name,
                                            rs_condiction.trim_start_matches("=")
                                        ));
                                    } else if rs_condiction.contains(" ") {
                                        let subs: Vec<&str> =
                                            rs_condiction.split_whitespace().collect();
                                        let mut sb = "(".to_string();
                                        let subs: Vec<String> = subs
                                            .iter()
                                            .map(|sub| {
                                                format!(
                                                    "{}.{} like '%{}%'",
                                                    table_name,
                                                    column_name,
                                                    sub.trim()
                                                )
                                            })
                                            .collect();
                                        let list_sql = subs.join(" or ");
                                        sb.push_str(&list_sql);
                                        sb.push_str(")");

                                        list.push(sb);
                                    } else {
                                        list.push(format!(
                                            "{}.{} = '{}'",
                                            table_name, column_name, rs_condiction
                                        ));
                                    }
                                }
                            }
                            Value::Array(array) => {
                                let mut limit: Vec<String> = Vec::new();
                                for obj in array {
                                    if obj.is_number() {
                                        limit.push(obj.to_string());
                                    } else if obj.is_string() {
                                        let s =
                                            obj.as_str().unwrap().to_string().replace("'", "''");
                                        limit.push(format!("'{}'", s));
                                    }
                                }

                                let in_list = limit.join(",");

                                list.push(format!(
                                    "{}.{} IN ({})",
                                    table_name, column_name, in_list
                                ))
                            }
                            Value::Object(obj) => {
                                let ty = obj.get("type");
                                if Some(&Value::String("between".to_string())) == ty {
                                    let begin = obj.get("begin");
                                    let end = obj.get("end");
                                    if let Some(Value::Number(begin)) = begin {
                                        list.push(format!(
                                            "{}.{} >= {}",
                                            table_name, column_name, begin
                                        ));
                                    }
                                    if let Some(Value::Number(end)) = end {
                                        list.push(format!(
                                            "{}.{} <= {}",
                                            table_name, column_name, end
                                        ));
                                    }
                                }
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
        let mut list = Vec::new();
        list
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
            let mut insert_init_column: Vec<String> = vec![
                "id",
                "ad_client_id",
                "ad_org_id",
                "ownerid",
                "modifiered",
                "creationdate",
                "modifieddate",
            ]
            .iter_mut()
            .map(|&mut e| e.to_string())
            .collect();
            list.append(&mut insert_init_column);

            for column in self.data.iter().filter(|c| self.is_insert_column(c)) {
                // 表单使用序号生成器
                if column.obtainmanner == Obtainmanner::SheetNo.to_string() {
                    if !column.sequencename.is_empty() {
                        list.push(column.dbname.to_string());
                        continue;
                    }
                }
                list.push(column.dbname.to_string());
            }
        }
        list
    }

    fn get_values(&self) -> Vec<String> {
        let mut list = Vec::new();
        if self.get_statement_type() == StatementType::INSERT {
            let mut insert_init_column: Vec<String> = vec![
                self.args.id.unwrap().to_string(),
                "37".to_string(),
                "27".to_string(),
                self.user_id.to_string(),
                self.user_id.to_string(),
                "sysdate".to_string(),
                "sysdate".to_string(),
            ];
            list.append(&mut insert_init_column);
            for column in self.data.iter().filter(|c| self.is_insert_column(c)) {
                if column.obtainmanner == Obtainmanner::SheetNo.to_string() {
                    list.push(format!("get_sequenceno('{}', 37)", column.sequencename));
                    continue;
                }
                let condition = column
                    .value
                    .clone()
                    .unwrap_or(Value::String(column.default_value.to_string()));

                match condition {
                    Value::Null => {
                        list.push("null".to_string());
                    }
                    Value::Bool(_) => todo!(),
                    Value::Number(num) => {
                        list.push(num.to_string());
                    }
                    Value::String(s) => {
                        if s.is_empty() {
                            list.push("null".to_string());
                        } else if !s.starts_with("0") && s.chars().all(|c| c.is_ascii_digit()) {
                            list.push(s.to_string());
                        } else if column.ref_table_id.is_some()
                            && column.ref_table.is_some()
                            && vec![Obtainmanner::Object, Obtainmanner::Operate].contains(&column.obtainmanner.clone().into()) {
                            // 支持dk 录入
                            let dk = column.ref_table.clone().unwrap();
                            list.push(format!(
                                "(select id from {} where {} = '{}')",
                                dk.table_name,
                                dk.dk_column,
                                s.to_string()
                            ));

                        } else {
                            list.push(format!("'{}'", s.replace("'", "''")));
                        }
                    }
                    _ => {}
                    // Value::Array(_) => todo!(),
                    // Value::Object(_) => todo!(),
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
                if column.mask.chars().nth(self.args.max_idx.unwrap_or(0)) == Some('1') {
                    if let Some(value) = &column.value {
                        match value {
                            Value::Number(num) => list.push(format!("{} = {}", column.dbname, num)),
                            Value::String(s) => {
                                list.push(format!("{} = '{}'", column.dbname, s.replace("'", "''")))
                            }
                            _ => {}
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
        let count = 10000;
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
