use serde_json::Value;

use crate::engine::Predicate;

use super::helpers::sanitize_filter_value;

pub(crate) fn string_predicates(target: &str, value: &str) -> Vec<Predicate> {
    let sanitized = sanitize_filter_value(value).replace('"', "");
    if sanitized.starts_with('=') {
        return vec![Predicate::eq(target.to_string(), sanitized.trim_start_matches('='))];
    }

    if sanitized.contains(' ') {
        let terms: Vec<String> = sanitized
            .split_whitespace()
            .map(|term| format!("%{}%", term.trim()))
            .collect();
        let sql = format!(
            "({})",
            terms
                .iter()
                .map(|_| format!("{} LIKE ?", target))
                .collect::<Vec<_>>()
                .join(" OR ")
        );
        return vec![Predicate::custom(
            sql,
            terms.into_iter().map(Value::String).collect(),
        )];
    }

    vec![Predicate::eq(target.to_string(), sanitized)]
}

pub(crate) fn object_predicates(
    target: &str,
    object: &serde_json::Map<String, Value>,
) -> Vec<Predicate> {
    let mut list = Vec::new();
    if object.get("type") == Some(&Value::String("between".to_string())) {
        if let Some(begin) = object.get("begin") {
            list.push(Predicate::gte(target.to_string(), begin.clone()));
        }
        if let Some(end) = object.get("end") {
            list.push(Predicate::lte(target.to_string(), end.clone()));
        }
    }
    list
}
