use crate::metadata::MetadataId;

#[derive(Debug)]
pub enum ExecutionError {
    DatasourceNotFound(MetadataId),
    DatasourceRegistration(String),
    Sqlx(sqlx::Error),
    Planning(String),
    Permission(String),
    Mapping(String),
}

impl From<sqlx::Error> for ExecutionError {
    fn from(value: sqlx::Error) -> Self {
        ExecutionError::Sqlx(value)
    }
}
