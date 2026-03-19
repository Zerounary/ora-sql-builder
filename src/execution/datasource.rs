use std::collections::HashMap;
use std::sync::{Arc, Once, RwLock};

use serde_json::Value;
use sqlx::any::AnyPoolOptions;
use sqlx::AnyPool;

use crate::metadata::{MetaDatasource, MetadataId};

use super::ExecutionError;

static ANY_DRIVERS: Once = Once::new();

#[derive(Default, Clone)]
pub struct DatasourceManager {
    pools: Arc<RwLock<HashMap<MetadataId, AnyPool>>>,
}

impl DatasourceManager {
    pub async fn register_datasource(
        &self,
        datasource: &MetaDatasource,
    ) -> Result<(), ExecutionError> {
        ensure_any_drivers();
        let max_connections = datasource
            .options
            .get("max_connections")
            .and_then(Value::as_u64)
            .unwrap_or(1);
        let pool = AnyPoolOptions::new()
            .max_connections(max_connections.try_into().unwrap_or(u32::MAX))
            .connect(&datasource.connection_uri)
            .await
            .map_err(|error| ExecutionError::DatasourceRegistration(error.to_string()))?;
        self.pools
            .write()
            .map_err(|_| {
                ExecutionError::DatasourceRegistration(
                    "failed to acquire datasource registry for writing".to_string(),
                )
            })?
            .insert(datasource.id, pool);
        Ok(())
    }

    pub fn get_pool(&self, datasource_id: MetadataId) -> Result<AnyPool, ExecutionError> {
        self.pools
            .read()
            .map_err(|_| {
                ExecutionError::DatasourceRegistration(
                    "failed to acquire datasource registry for reading".to_string(),
                )
            })?
            .get(&datasource_id)
            .cloned()
            .ok_or(ExecutionError::DatasourceNotFound(datasource_id))
    }

    pub async fn health_check(&self, datasource_id: MetadataId) -> Result<(), ExecutionError> {
        let pool = self.get_pool(datasource_id)?;
        sqlx::query("SELECT 1").execute(&pool).await?;
        Ok(())
    }
}

fn ensure_any_drivers() {
    ANY_DRIVERS.call_once(sqlx::any::install_default_drivers);
}
