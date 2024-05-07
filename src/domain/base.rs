use std::collections::HashSet;

use chrono::Utc;
use serde::{Deserialize, Serialize};

pub trait Model {
    fn id(&self) -> String;
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct BaseModel {
    pub id: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub deleted_at: u64,
    pub version: u64,
}

impl BaseModel {
    pub fn new(id: String) -> Self {
        BaseModel {
            id,
            created_at: Utc::now().timestamp() as u64,
            updated_at: Utc::now().timestamp() as u64,
            version: 0,
            deleted_at: 0,
        }
    }

    pub fn delete(&mut self) {
        self.deleted_at = Utc::now().timestamp() as u64;
    }

    pub fn fake() -> Self {
        BaseModel {
            id: "fake".to_string(),
            created_at: 0,
            updated_at: 0,
            deleted_at: 0,
            version: 0,
        }
    }
}

/// compare the difference between the target_ids and the models.
/// if the target_ids contains the id that is not in the models, return the invalid ids
pub fn difference_ids<T: Model>(target_ids: &Vec<String>, models: &Vec<T>) -> Vec<String> {
    let model_ids = models
        .iter()
        .map(|item| item.id())
        .collect::<HashSet<String>>();

    let invalid_ids: Vec<String> = target_ids
        .iter()
        .map(|id| id.to_string())
        .collect::<HashSet<String>>()
        .difference(&model_ids)
        .into_iter()
        .map(|id| id.to_string())
        .collect();

    invalid_ids
}
