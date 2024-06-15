use axum::Form;
use serde::{Deserialize, Serialize};

use crate::actors::{fetcher, rbac};

use super::BaseModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteItem {
    pub module: String,
    pub path: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Role {
    #[serde(flatten)]
    pub base: BaseModel,
    pub name: String,
    pub permissions: Vec<RouteItem>,
}

impl Role {
    pub fn new(id: String, name: String, permissions: Vec<RouteItem>) -> Self {
        Role {
            base: BaseModel::new(id),
            name,
            permissions,
        }
    }
}

impl fetcher::RBACRole for Role {
    fn to_casbin_policy(&self) -> Vec<Vec<String>> {
        let mut out: Vec<Vec<String>> = vec![];

        self.permissions
            .iter()
            .for_each(|p| out.push(vec![self.name.clone(), p.path.clone()]));

        out
    }
}
