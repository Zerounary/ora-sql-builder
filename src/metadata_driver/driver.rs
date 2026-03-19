use serde_json::Value;

use crate::engine::{
    BuiltQuery, DeleteBuilder, InsertBuilder, MetaSqlEngine, Predicate, SelectBuilder, SqlDialect,
    TableRef, UpdateBuilder,
};
use crate::metadata::{
    FieldInputKind, FieldSource, MetadataField, MetadataQueryRequest, SortDirection,
};
use crate::sql::StatementType;

use super::context::{build_link_context, LinkContext};
use super::filters::{object_predicates, predicate_from_filter_expr, string_predicates};
use super::helpers::{assignment_sql, push_unique, Assignment};

pub struct MetadataSqlDriver {
    engine: MetaSqlEngine,
    request: MetadataQueryRequest,
}

impl MetadataSqlDriver {
    pub fn new(request: MetadataQueryRequest) -> Self {
        Self {
            engine: MetaSqlEngine::default(),
            request,
        }
    }

    pub fn build(&self, dialect: &dyn SqlDialect) -> BuiltQuery {
        match self.request.statement_type {
            StatementType::SELECT => self.build_select(dialect),
            StatementType::INSERT => self.build_insert(dialect),
            StatementType::UPDATE => self.build_update(dialect),
            StatementType::DELETE => self.build_delete(dialect),
        }
    }

    fn build_select(&self, dialect: &dyn SqlDialect) -> BuiltQuery {
        let link_context = build_link_context(&self.request, &self.main_table_alias());
        let mut builder = SelectBuilder::new(self.main_table_ref(true));

        if !self.request.options.grouped {
            builder = builder.select(format!("{}.id AS \"id\"", self.main_table_alias()));
        }

        for relation in link_context.relations.iter().cloned() {
            builder = builder.relation(relation);
        }

        for field in self.selected_fields() {
            for projection in self.select_projections(field, &link_context) {
                builder = builder.select(projection);
            }
        }

        for predicate in self.select_predicates(&link_context) {
            builder = builder.predicate(predicate);
        }

        for predicate in self.having_predicates(&link_context) {
            builder = builder.having(predicate);
        }

        if self.request.options.grouped {
            for expression in self.group_by_expressions(&link_context) {
                builder = builder.group_by(expression.clone()).order_by(expression);
            }
        } else {
            for expression in self.order_by_expressions(&link_context) {
                builder = builder.order_by(expression);
            }
        }

        self.engine.build_select(dialect, builder)
    }

    fn build_insert(&self, dialect: &dyn SqlDialect) -> BuiltQuery {
        let mut builder = InsertBuilder::new(self.main_table_name());
        builder = builder
            .value("id", self.request.options.id.unwrap())
            .value("ad_client_id", self.request.options.client_id)
            .value("ad_org_id", self.request.options.org_id)
            .value("ownerid", self.request.user_id)
            .value("modifiered", self.request.user_id)
            .raw_value("creationdate", "sysdate")
            .raw_value("modifieddate", "sysdate");

        for field in self.insertable_fields() {
            if let Some(column_name) = field.source_column() {
                if field.input_kind == FieldInputKind::Sequence {
                    if let Some(sequence_name) = &field.sequence_name {
                        builder = builder.raw_value(
                            column_name,
                            format!(
                                "get_sequenceno('{}', {})",
                                sequence_name, self.request.options.client_id
                            ),
                        );
                    }
                    continue;
                }

                let value = field
                    .value
                    .clone()
                    .or_else(|| field.default_value.clone())
                    .unwrap_or(Value::Null);
                builder = match assignment_sql(field, value) {
                    Assignment::Param(value) => builder.value(column_name, value),
                    Assignment::Raw(sql) => builder.raw_value(column_name, sql),
                };
            }
        }

        self.engine.build_insert(dialect, builder)
    }

    fn build_update(&self, dialect: &dyn SqlDialect) -> BuiltQuery {
        let mut builder = UpdateBuilder::new(self.main_table_name())
            .set("modifierid", self.request.user_id)
            .set_raw("modifieddate", "sysdate");

        for field in self.writable_fields() {
            if let Some(column_name) = field.source_column() {
                if let Some(value) = &field.value {
                    builder = match assignment_sql(field, value.clone()) {
                        Assignment::Param(value) => builder.set(column_name, value),
                        Assignment::Raw(sql) => builder.set_raw(column_name, sql),
                    };
                }
            }
        }

        if let Some(table_filter) = &self.request.options.table_filter {
            if !table_filter.is_empty() {
                builder = builder.predicate(Predicate::raw(table_filter.clone()));
            }
        }

        if let Some(id) = self.request.options.id {
            builder = builder.predicate(Predicate::eq("id", id));
        }

        self.engine.build_update(dialect, builder)
    }

    fn build_delete(&self, dialect: &dyn SqlDialect) -> BuiltQuery {
        let mut builder = DeleteBuilder::new(self.main_table_name());

        if let Some(table_filter) = &self.request.options.table_filter {
            if !table_filter.is_empty() {
                builder = builder.predicate(Predicate::raw(table_filter.clone()));
            }
        }

        if let Some(id) = self.request.options.id {
            builder = builder.predicate(Predicate::eq("id", id));
        }

        self.engine.build_delete(dialect, builder)
    }

    fn selected_fields(&self) -> impl Iterator<Item = &MetadataField> {
        self.request
            .fields
            .iter()
            .filter(|field| field.access.allows(self.request.options.mask_index))
    }

    fn insertable_fields(&self) -> impl Iterator<Item = &MetadataField> {
        self.request.fields.iter().filter(|field| {
            let is_background_field = !field.nullable
                && field
                    .source_column()
                    .map(|column| !column.eq_ignore_ascii_case("id"))
                    .unwrap_or(false)
                && field.input_kind != FieldInputKind::Trigger;
            let is_plain_column = matches!(field.source, FieldSource::Column(_));
            (field.access.allows(self.request.options.mask_index) || is_background_field)
                && is_plain_column
        })
    }

    fn writable_fields(&self) -> impl Iterator<Item = &MetadataField> {
        self.request.fields.iter().filter(|field| {
            field.access.allows(self.request.options.mask_index)
                && field.value.is_some()
                && matches!(field.source, FieldSource::Column(_))
        })
    }

    fn select_projections(&self, field: &MetadataField, link_context: &LinkContext) -> Vec<String> {
        let output_name = field.output_name();
        match &field.source {
            FieldSource::Column(column) => {
                if let Some(lookup) = &field.lookup {
                    vec![
                        format!(
                            "{}.{} AS \"{}\"",
                            field.current_table, column, output_name
                        ),
                        format!(
                            "(select {} as dk from {} x where id = {}.{}) as \"{}.dk\"",
                            lookup.display_column,
                            lookup.table,
                            field.current_table,
                            column,
                            output_name
                        ),
                    ]
                } else {
                    vec![format!(
                        "{}.{} AS \"{}\"",
                        field.current_table, column, output_name
                    )]
                }
            }
            FieldSource::Qualified(expression) | FieldSource::Formula(expression) => {
                vec![format!("{} AS \"{}\"", expression, output_name)]
            }
            FieldSource::Linked(link) => vec![format!(
                "{}.{} AS \"{}\"",
                link_context.alias_for(link),
                link.target_column,
                output_name
            )],
        }
    }

    fn select_predicates(&self, link_context: &LinkContext) -> Vec<Predicate> {
        let mut predicates = Vec::new();

        if let Some(table_filter) = &self.request.options.table_filter {
            if !table_filter.is_empty() {
                predicates.push(Predicate::raw(table_filter.clone()));
            }
        }

        predicates.extend(self.request.filters.iter().map(|filter| {
            predicate_from_filter_expr(filter, &|field| self.named_filter_target(field, link_context))
        }));

        for field in self.request.fields.iter().filter(|field| field.value.is_some()) {
            let target = self.filter_target(field, link_context);
            match field.value.as_ref().unwrap() {
                Value::Null => {}
                Value::Bool(flag) => {
                    predicates.push(Predicate::eq(target, if *flag { "Y" } else { "N" }))
                }
                Value::Number(number) => predicates.push(Predicate::eq(target, number.clone())),
                Value::String(text) => predicates.extend(string_predicates(&target, text)),
                Value::Array(array) => predicates.push(Predicate::in_list(target, array.clone())),
                Value::Object(object) => predicates.extend(object_predicates(&target, object)),
            }
        }

        if let Some(id) = self.request.options.id {
            predicates.push(Predicate::eq(format!("{}.id", self.main_table_alias()), id));
        }

        predicates
    }

    fn having_predicates(&self, link_context: &LinkContext) -> Vec<Predicate> {
        self.request
            .having
            .iter()
            .map(|filter| {
                predicate_from_filter_expr(filter, &|field| self.named_filter_target(field, link_context))
            })
            .collect()
    }

    fn group_by_expressions(&self, link_context: &LinkContext) -> Vec<String> {
        let mut list = Vec::new();
        for field in self.selected_fields() {
            match &field.source {
                FieldSource::Column(column) => {
                    push_unique(&mut list, format!("{}.{}", field.current_table, column));
                }
                FieldSource::Qualified(expression) => {
                    push_unique(&mut list, expression.clone());
                }
                FieldSource::Linked(link) => {
                    push_unique(
                        &mut list,
                        format!("{}.{}", link_context.alias_for(link), link.target_column),
                    );
                }
                FieldSource::Formula(_) => {}
            }
        }
        list
    }

    fn order_by_expressions(&self, link_context: &LinkContext) -> Vec<String> {
        let mut list = Vec::new();
        for field in self.request.fields.iter().filter(|field| field.sort.is_some()) {
            let direction = match field.sort.as_ref().unwrap() {
                SortDirection::Asc => "asc",
                SortDirection::Desc => "desc",
            };
            let expression = match &field.source {
                FieldSource::Column(column) => format!("{}.{}", field.current_table, column),
                FieldSource::Qualified(expression) | FieldSource::Formula(expression) => {
                    expression.clone()
                }
                FieldSource::Linked(link) => {
                    format!("{}.{}", link_context.alias_for(link), link.target_column)
                }
            };
            list.push(format!("{} {} nulls first", expression, direction));
        }
        list
    }

    fn filter_target(&self, field: &MetadataField, link_context: &LinkContext) -> String {
        match &field.source {
            FieldSource::Column(column) => format!("{}.{}", field.current_table, column),
            FieldSource::Qualified(expression) | FieldSource::Formula(expression) => expression.clone(),
            FieldSource::Linked(link) => {
                format!("{}.{}", link_context.alias_for(link), link.target_column)
            }
        }
    }

    fn named_filter_target(&self, field_name: &str, link_context: &LinkContext) -> String {
        if let Some(field) = self.request.fields.iter().find(|candidate| {
            candidate.output_alias.as_deref() == Some(field_name)
                || candidate.output_name() == field_name
                || candidate.source_column() == Some(field_name)
        }) {
            return self.filter_target(field, link_context);
        }

        field_name.to_string()
    }

    fn main_table_ref(&self, alias_real_table: bool) -> TableRef {
        let main_table_name = self.main_table_name();
        if alias_real_table && self.main_field().real_table.is_some() {
            TableRef::new(main_table_name).alias(self.main_table_alias())
        } else {
            TableRef::new(main_table_name)
        }
    }

    fn main_table_name(&self) -> String {
        self.main_field()
            .real_table
            .clone()
            .unwrap_or_else(|| self.main_field().current_table.clone())
    }

    fn main_table_alias(&self) -> String {
        self.main_field().current_table.clone()
    }

    fn main_field(&self) -> &MetadataField {
        self.request
            .fields
            .first()
            .expect("metadata request requires at least one field")
    }
}
