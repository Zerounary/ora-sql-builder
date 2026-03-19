use std::collections::HashMap;

use crate::engine::{JoinType, Relation, TableRef};
use crate::metadata::{FieldSource, LinkReference, MetadataQueryRequest};

pub(crate) struct LinkContext {
    pub(crate) relations: Vec<Relation>,
    aliases: HashMap<String, String>,
}

impl LinkContext {
    pub(crate) fn alias_for(&self, link: &LinkReference) -> String {
        let key = link
            .steps
            .iter()
            .map(|step| step.foreign_key.clone())
            .collect::<Vec<_>>()
            .join(";");
        self.aliases.get(&key).cloned().unwrap()
    }
}

pub(crate) fn build_link_context(request: &MetadataQueryRequest, main_alias: &str) -> LinkContext {
    let mut aliases: HashMap<String, String> = HashMap::new();
    let mut relations = Vec::new();
    let mut alias_index = 0;

    for field in &request.fields {
        let FieldSource::Linked(link) = &field.source else {
            continue;
        };

        let mut left_alias = main_alias.to_string();
        let mut prefix = Vec::new();
        for step in &link.steps {
            prefix.push(step.foreign_key.clone());
            let key = prefix.join(";");
            let alias = if let Some(existing) = aliases.get(&key) {
                existing.clone()
            } else {
                alias_index += 1;
                let alias = format!("a{}", alias_index);
                relations.push(Relation::new(
                    JoinType::Left,
                    left_alias.clone(),
                    step.foreign_key.clone(),
                    TableRef::new(step.table.clone()).alias(alias.clone()),
                    "id",
                ));
                aliases.insert(key.clone(), alias.clone());
                alias
            };
            left_alias = alias;
        }
    }

    LinkContext { relations, aliases }
}
