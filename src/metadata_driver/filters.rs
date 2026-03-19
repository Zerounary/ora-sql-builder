use serde_json::Value;

use crate::engine::Predicate;
use crate::metadata::{MetadataFilter, MetadataFilterExpr};

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

pub(crate) fn predicate_from_filter_expr<F>(
    filter: &MetadataFilterExpr,
    resolve_target: &F,
) -> Predicate
where
    F: Fn(&str) -> String,
{
    match filter {
        MetadataFilterExpr::Field { field, filter } => {
            field_predicate(resolve_target(field), filter)
        }
        MetadataFilterExpr::Exists { sql, params } => Predicate::exists(sql.clone(), params.clone()),
        MetadataFilterExpr::Custom { sql, params } => Predicate::custom(sql.clone(), params.clone()),
        MetadataFilterExpr::Raw(sql) => Predicate::raw(sql.clone()),
        MetadataFilterExpr::And(filters) => Predicate::and(
            filters
                .iter()
                .map(|filter| predicate_from_filter_expr(filter, resolve_target))
                .collect(),
        ),
        MetadataFilterExpr::Or(filters) => Predicate::or(
            filters
                .iter()
                .map(|filter| predicate_from_filter_expr(filter, resolve_target))
                .collect(),
        ),
        MetadataFilterExpr::Not(filter) => {
            Predicate::not(predicate_from_filter_expr(filter, resolve_target))
        }
    }
}

fn field_predicate(target: String, filter: &MetadataFilter) -> Predicate {
    match filter {
        MetadataFilter::Eq(value) => Predicate::eq(target, value.clone()),
        MetadataFilter::Ne(value) => Predicate::ne(target, value.clone()),
        MetadataFilter::Gt(value) => Predicate::gt(target, value.clone()),
        MetadataFilter::Gte(value) => Predicate::gte(target, value.clone()),
        MetadataFilter::Lt(value) => Predicate::lt(target, value.clone()),
        MetadataFilter::Lte(value) => Predicate::lte(target, value.clone()),
        MetadataFilter::Like(value) => Predicate::like(target, value.clone()),
        MetadataFilter::In(values) => Predicate::in_list(target, values.clone()),
        MetadataFilter::Between { lower, upper } => {
            Predicate::between(target, lower.clone(), upper.clone())
        }
        MetadataFilter::IsNull => Predicate::is_null(target),
        MetadataFilter::IsNotNull => Predicate::is_not_null(target),
    }
}
