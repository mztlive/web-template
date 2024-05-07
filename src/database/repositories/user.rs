use mongodb::{bson::doc, Database};

use crate::{
    actors::rbac,
    domain::{role::Role, user::User},
};

use super::collection_names::USER;

use async_trait::async_trait;

use futures_util::StreamExt;

pub struct UserRepository {
    pub coll_name: String,
}

impl UserRepository {
    pub fn new() -> Self {
        UserRepository {
            coll_name: USER.to_string(),
        }
    }
}

#[async_trait]
impl rbac::RBACUserFetcher for UserRepository {
    async fn find_all(
        &self,
        database: &Database,
    ) -> std::result::Result<Vec<Box<dyn rbac::RBACUser>>, String> {
        let collection = database.collection::<User>(self.coll_name.as_str());
        let mut cursor = collection
            .find(
                doc! {
                    "deleted_at": 0
                },
                None,
            )
            .await
            .map_err(|e| e.to_string())?;

        let mut users: Vec<Box<dyn rbac::RBACUser>> = vec![];

        while let Some(result) = cursor.next().await {
            let user = result.map_err(|e| e.to_string())?;
            users.push(Box::new(user));
        }

        Ok(users)
    }
}
