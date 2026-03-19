use crate::metadata::{DatabaseKind, MetadataFilterExpr, PolicyKind, PrimaryKeyStrategy, RelationKind};

pub(super) fn database_kind_code(kind: &DatabaseKind) -> String {
    match kind {
        DatabaseKind::MySql => "mysql".to_string(),
        DatabaseKind::Postgres => "postgres".to_string(),
        DatabaseKind::Oracle => "oracle".to_string(),
        DatabaseKind::SqlServer => "sqlserver".to_string(),
        DatabaseKind::Sqlite => "sqlite".to_string(),
        DatabaseKind::Custom(value) => value.clone(),
    }
}

pub(super) fn primary_key_strategy_code(strategy: &PrimaryKeyStrategy) -> String {
    match strategy {
        PrimaryKeyStrategy::Manual => "manual".to_string(),
        PrimaryKeyStrategy::AutoIncrement => "auto_increment".to_string(),
        PrimaryKeyStrategy::Sequence(name) => format!("sequence:{}", name),
        PrimaryKeyStrategy::Snowflake => "snowflake".to_string(),
        PrimaryKeyStrategy::Uuid => "uuid".to_string(),
    }
}

pub(super) fn relation_kind_code(kind: &RelationKind) -> String {
    match kind {
        RelationKind::OneToOne => "one_to_one".to_string(),
        RelationKind::OneToMany => "one_to_many".to_string(),
        RelationKind::ManyToOne => "many_to_one".to_string(),
        RelationKind::ManyToMany => "many_to_many".to_string(),
    }
}

pub(super) fn policy_kind_code(kind: &PolicyKind) -> String {
    match kind {
        PolicyKind::RowFilter => "row_filter".to_string(),
        PolicyKind::FieldMask => "field_mask".to_string(),
        PolicyKind::ImportGuard => "import_guard".to_string(),
        PolicyKind::ExportGuard => "export_guard".to_string(),
        PolicyKind::Custom(value) => value.clone(),
    }
}

pub(super) fn filter_expr_text(filter: &MetadataFilterExpr) -> String {
    format!("{:?}", filter)
}
