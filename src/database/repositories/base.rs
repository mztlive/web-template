use super::super::errors::Result;
use futures_util::StreamExt;
use mongodb::{bson::Document, Cursor};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Collection<T> {
    pub items: Vec<T>,
    pub total: i64,
}

/// validate page size
pub fn default_page_size() -> i64 {
    20
}

/// validate page size
pub fn default_page() -> i64 {
    1
}

pub enum NumberItemValueType {
    I64,
    I32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NumberItem {
    pub id: Option<String>,
    pub name: String,
    pub value: i64,
}

impl NumberItem {
    pub fn new(name: String, value: i64, id: Option<String>) -> Self {
        NumberItem { name, value, id }
    }

    /// returns a vector of [NumberItem] from a cursor
    ///
    /// Require a cursor document must with 2 fields: _id and count
    /// _id will be using as name and id for [NumberItem]
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    /// * access value error
    pub async fn vec_from_cursor(mut cursor: Cursor<Document>, v_type: NumberItemValueType) -> Result<Vec<NumberItem>> {
        let mut result = vec![];
        while let Some(item) = cursor.next().await {
            let item = item?;
            let name = item.get_str("_id")?;

            let count = match v_type {
                NumberItemValueType::I64 => item.get_i64("count")?,
                NumberItemValueType::I32 => item.get_i32("count")? as i64,
            };

            result.push(NumberItem::new(name.to_string(), count, Some(name.to_string())));
        }

        Ok(result)
    }
}

pub async fn cursor_to_vec<T>(mut cursor: Cursor<T>) -> Result<Vec<T>>
where
    Cursor<T>: futures::stream::StreamExt<Item = std::result::Result<T, mongodb::error::Error>>,
{
    let mut result = vec![];
    while let Some(item) = cursor.next().await {
        result.push(item?);
    }

    Ok(result)
}
