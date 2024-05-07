use mongodb::{bson::doc, Database};

use crate::{actors::rbac, domain::role::Role};

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
impl rbac::RBACRoleFetcher for RoleRepository {
    async fn find_all(
        &self,
        database: &Database,
    ) -> std::result::Result<Vec<Box<dyn rbac::RBACRole>>, String> {
        let mut items = database
            .collection::<Role>(self.coll_name.as_str())
            .find(
                doc! {
                    "deleted_at": 0
                },
                None,
            )
            .await
            .map_err(|e| e.to_string())?;

        let mut out: Vec<Box<dyn rbac::RBACRole>> = vec![];

        while let Some(item) = items.next().await {
            let item = item.map_err(|e| e.to_string())?;
            out.push(Box::new(item));
        }

        Ok(out)
    }
}
