use serde::{Deserialize, Serialize};

use crate::actors::{fetcher, rbac};

use super::{common::Secret, BaseModel};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct User {
    #[serde(flatten)]
    pub base: BaseModel,
    pub secret: Secret,
    pub name: String,
    pub age: u8,
    pub avatar: String,
    pub is_active: bool,
    pub role_name: String,
}

impl fetcher::RBACUser for User {
    fn account(&self) -> String {
        self.name.clone()
    }

    fn role_name(&self) -> String {
        self.role_name.clone()
    }
}
