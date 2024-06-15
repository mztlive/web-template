use mongodb::{bson::doc, Database};
use serde::de::DeserializeOwned;

use crate::{
    actors::fetcher::{self, Error, RBACRole},
    domain::role::Role,
};

use super::collection_names::ROLE;

use async_trait::async_trait;

use futures_util::StreamExt;

pub struct RoleRepository {
    pub coll_name: String,
}

impl RoleRepository {
    pub fn new() -> Self {
        RoleRepository {
            coll_name: ROLE.to_string(),
        }
    }
}

#[async_trait]
impl fetcher::RBACRoleFetcher for RoleRepository {
    async fn find_all(&self, database: &Database) -> Result<Vec<Box<dyn RBACRole>>, Error> {
        let mut items = database
            .collection::<Role>(self.coll_name.as_str())
            .find(
                doc! {
                    "deleted_at": 0
                },
                None,
            )
            .await?;

        let mut out: Vec<Box<dyn RBACRole>> = vec![];

        while let Some(item) = items.next().await {
            let item = item?;
            out.push(Box::new(item));
        }

        Ok(out)
    }
}
