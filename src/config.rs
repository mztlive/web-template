use mongodb::Collection;
use serde::Deserialize;
use tokio::fs;

use crate::{
    actors::{id_gen::IDGeneratorHandler, rbac::RbacActorHandler},
    jwt::Engine,
};
#[derive(Clone)]
pub struct AppState {
    pub client: mongodb::Client,
    pub db: mongodb::Database,
    pub config: AppConfig,
    pub id_gen: IDGeneratorHandler,
    pub jwt: Engine,
    pub rbac: RbacActorHandler,
}

impl AppState {
    pub fn get_collection<T>(&self, collection_name: &str) -> Collection<T> {
        self.client
            .database(&self.config.database.db_name)
            .collection(collection_name)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Parse Error: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Deserialize, Clone)]
pub struct Database {
    pub uri: String,
    pub db_name: String,
}

#[derive(Deserialize, Clone)]
pub struct AppConfig {
    pub database: Database,
    pub secret: String,
    pub statistic_host: String,
}

impl AppConfig {
    pub fn get_statistic_url(&self, path: &str) -> String {
        let path = path.trim_start_matches("./");
        let path = path.trim_start_matches("/upload");
        format!("{}/{}", self.statistic_host, path)
    }
}

/// Load the configuration from a file
///
/// # Errors
///
/// This function will return an error if the file cannot be read or if the file cannot be parsed.
pub async fn load_config(path: &str) -> Result<AppConfig, Error> {
    let content = fs::read_to_string(path).await?;
    let cfg = toml::from_str(&content)?;

    Ok(cfg)
}
