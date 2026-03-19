#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CapabilityMask(pub(crate) Vec<bool>);

impl CapabilityMask {
    pub fn allows(&self, index: usize) -> bool {
        self.0.get(index).copied().unwrap_or(false)
    }
}

impl From<&str> for CapabilityMask {
    fn from(value: &str) -> Self {
        Self(value.chars().map(|ch| ch == '1').collect())
    }
}

impl From<String> for CapabilityMask {
    fn from(value: String) -> Self {
        CapabilityMask::from(value.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum FieldInputKind {
    #[default]
    Text,
    Lookup,
    Ignored,
    Operation,
    Sequence,
    Trigger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

pub type MetadataId = i64;
