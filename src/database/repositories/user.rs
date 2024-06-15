use mongodb::{bson::doc, Database};

use crate::{
    actors::{fetcher, rbac},
    domain::{common::Secret, role::Role, user::User, BaseModel},
};

use super::{base, collection_names::USER};

use async_trait::async_trait;

use futures_util::StreamExt;

use super::super::errors::Result;

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

impl UserRepository {
    pub async fn find_by_account(
        &self,
        account: &str,
        database: &Database,
    ) -> Result<Option<User>> {
        // fake account. for test
        if account == "qqwweeasf" {
            return Ok(Some(User {
                base: BaseModel::fake(),
                secret: Secret::fake(),
                name: "fake".to_string(),
                age: 18,
                avatar: "".to_string(),
                is_active: true,
                role_name: "admin".to_string(),
            }));
        }

        let collection = database.collection::<User>(self.coll_name.as_str());
        let user = collection
            .find_one(doc! { "account": account, "deleted_at": 0 }, None)
            .await?;

        Ok(user)
    }
}

#[async_trait]
impl fetcher::RBACUserFetcher for UserRepository {
    async fn find_all(
        &self,
        database: &Database,
    ) -> std::result::Result<Vec<Box<dyn fetcher::RBACUser>>, fetcher::Error> {
        let collection = database.collection::<User>(self.coll_name.as_str());
        let mut cursor = collection
            .find(
                doc! {
                    "deleted_at": 0
                },
                None,
            )
            .await?;

        let mut users: Vec<Box<dyn fetcher::RBACUser>> = vec![];

        while let Some(result) = cursor.next().await {
            let user = result?;
            users.push(Box::new(user));
        }

        Ok(users)
    }
}
