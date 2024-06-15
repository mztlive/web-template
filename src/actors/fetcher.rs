use async_trait::async_trait;
use mongodb::Database;
use serde::{Deserialize, Serialize};

pub trait RBACRole: Send {
    fn to_casbin_policy(&self) -> Vec<Vec<String>>;
}

pub trait RBACUser: Send {
    fn account(&self) -> String;

    fn role_name(&self) -> String;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Fetcher Error from MongoDB: {0}")]
    DatabaseError(#[from] mongodb::error::Error),
}

/// fetch all roles from database
///
/// # Errors
///
/// - DatabaseError: if failed to fetch all roles from database
#[async_trait]
pub trait RBACRoleFetcher: Send {
    async fn find_all(&self, database: &Database) -> Result<Vec<Box<dyn RBACRole>>, Error>;
}

/// fetch all users from database
///
/// # Errors
///
/// - DatabaseError: if failed to fetch all users from database
#[async_trait]
pub trait RBACUserFetcher: Send {
    async fn find_all(&self, database: &Database) -> Result<Vec<Box<dyn RBACUser>>, Error>;
}
