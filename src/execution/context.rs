use crate::metadata::MetaDatasource;

pub struct ExecutionContext<'a> {
    pub manager: &'a super::DatasourceManager,
    pub datasource: &'a MetaDatasource,
    pub options: ExecutionOptions,
}

impl<'a> ExecutionContext<'a> {
    pub fn new(
        manager: &'a super::DatasourceManager,
        datasource: &'a MetaDatasource,
        options: ExecutionOptions,
    ) -> Self {
        Self {
            manager,
            datasource,
            options,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionMode {
    Query,
    Write,
    Delete,
    Schema,
    Metadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionOptions {
    pub mode: ExecutionMode,
    pub dry_run: bool,
    pub transactional: bool,
    pub max_rows: Option<usize>,
}

impl Default for ExecutionOptions {
    fn default() -> Self {
        Self {
            mode: ExecutionMode::Query,
            dry_run: false,
            transactional: false,
            max_rows: None,
        }
    }
}
