use serde_json::{json, Value};

use crate::metadata::MetadataField;

pub(crate) enum Assignment {
    Param(Value),
    Raw(String),
}

pub(crate) fn assignment_sql(field: &MetadataField, value: Value) -> Assignment {
    match value {
        Value::Null => Assignment::Param(Value::Null),
        Value::Bool(flag) => Assignment::Param(Value::String(if flag { "Y" } else { "N" }.to_string())),
        Value::Number(number) => Assignment::Param(Value::Number(number)),
        Value::String(text) => {
            if text.is_empty() {
                Assignment::Param(Value::Null)
            } else if let Some(lookup) = &field.lookup {
                if is_numeric_string_without_leading_zero(&text) {
                    Assignment::Param(json!(text.parse::<i64>().unwrap()))
                } else {
                    Assignment::Raw(format!(
                        "(select id from {} where {} = '{}')",
                        lookup.table,
                        lookup.display_column,
                        text.replace("'", "''")
                    ))
                }
            } else if is_numeric_string_without_leading_zero(&text) {
                Assignment::Param(json!(text.parse::<i64>().unwrap()))
            } else {
                Assignment::Param(Value::String(text))
            }
        }
        other => Assignment::Param(other),
    }
}

pub(crate) fn push_unique(list: &mut Vec<String>, item: String) {
    if !list.contains(&item) {
        list.push(item);
    }
}

pub(crate) fn sanitize_filter_value(value: &str) -> String {
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

pub(crate) fn is_numeric_string_without_leading_zero(value: &str) -> bool {
    !value.starts_with('0') && value.chars().all(|ch| ch.is_ascii_digit())
}
