use super::{base::cursor_to_vec, collection_names};
use futures_util::stream::StreamExt;

use mongodb::{
    bson::{doc, to_bson, Document},
    options::FindOptions,
    Database,
};

use crate::database::errors::{Error, Result};

use super::Collection;

pub trait IFilter {
    fn to_doc(&self) -> Document;
}

pub trait IPaginator {
    fn skip(&self) -> u64;

    fn limit(&self) -> i64;
}

#[macro_export]
macro_rules! impl_repository {
    ($repo:ident, $struct_name:ty, $collection:expr) => {
        impl $repo {
            pub async fn create(&self, entity: &$struct_name, database: &Database) -> Result<()> {
                database
                    .collection::<$struct_name>($collection)
                    .insert_one(entity, None)
                    .await?;

                Ok(())
            }

            pub async fn find_by_id(
                &self,
                id: &str,
                database: &Database,
            ) -> Result<Option<$struct_name>> {
                let entity = database
                    .collection::<$struct_name>($collection)
                    .find_one(doc! { "id": id, "deleted_at":0 }, None)
                    .await?;

                Ok(entity)
            }

            pub async fn find_by_ids(
                &self,
                ids: &[String],
                database: &Database,
            ) -> Result<Vec<$struct_name>> {
                let mut cursor = database
                    .collection::<$struct_name>($collection)
                    .find(doc! { "id":  {"$in": ids}, "deleted_at":0 }, None)
                    .await?;

                let mut entities = vec![];
                while let Some(entity) = cursor.next().await {
                    entities.push(entity?);
                }

                Ok(entities)
            }

            pub async fn update(&self, entity: &$struct_name, database: &Database) -> Result<()> {
                let previous_version = entity.base.version;
                let next_version = previous_version + 1;
                let bson = to_bson(entity)?;

                // add version
                let mut doc = bson.as_document().unwrap().clone();
                doc.insert("version", next_version as i64);

                let result = database
                    .collection::<$struct_name>($collection)
                    .update_one(
                        doc! {
                            "id": entity.base.id.clone(),
                            // cas lock
                            "version": previous_version as i64,
                        },
                        doc! {
                            "$set": doc
                        },
                        None,
                    )
                    .await?;

                if result.modified_count == 0 {
                    return Err(Error::OptimisticLockingError);
                }

                Ok(())
            }

            pub async fn update_with_session(
                &self,
                entity: &$struct_name,
                database: &Database,
                session: &mut mongodb::ClientSession,
            ) -> Result<()> {
                let previous_version = entity.base.version;
                let next_version = previous_version + 1;
                let bson = to_bson(entity)?;

                // add version
                let mut doc = bson.as_document().unwrap().clone();
                doc.insert("version", next_version as i64);

                let result = database
                    .collection::<$struct_name>($collection)
                    .update_one_with_session(
                        doc! {
                            "id": entity.base.id.clone(),
                            // cas lock
                            "version": previous_version as i64,
                        },
                        doc! {
                            "$set": doc
                        },
                        None,
                        session,
                    )
                    .await?;

                if result.modified_count == 0 {
                    return Err(Error::OptimisticLockingError);
                }

                Ok(())
            }

            pub async fn find_all(&self, database: &Database) -> Result<Vec<$struct_name>> {
                let cursor = database
                    .collection::<$struct_name>($collection)
                    .find(doc! {"deleted_at": 0}, None)
                    .await?;

                cursor_to_vec(cursor).await
            }

            pub async fn search<T>(
                &self,
                database: &Database,
                filter: &T,
            ) -> Result<Collection<$struct_name>>
            where
                T: IFilter + IPaginator,
            {
                let items = self.search_slice(database, filter).await?;
                let total = self.search_count(database, filter).await?;

                Ok(Collection {
                    items,
                    total: total as i64,
                })
            }

            pub async fn search_slice<T>(
                &self,
                database: &Database,
                filter: &T,
            ) -> Result<Vec<$struct_name>>
            where
                T: IFilter + IPaginator,
            {
                let find_options = FindOptions::builder()
                    .sort(doc! { "created_at": -1 })
                    .skip(Some(filter.skip()))
                    .limit(Some(filter.limit()))
                    .build();

                let cursor = database
                    .collection::<$struct_name>($collection)
                    .find(filter.to_doc(), find_options)
                    .await?;

                cursor_to_vec(cursor).await
            }

            pub async fn search_count<T>(&self, database: &Database, filter: &T) -> Result<u64>
            where
                T: IFilter,
            {
                let count = database
                    .collection::<$struct_name>($collection)
                    .count_documents(filter.to_doc(), None)
                    .await?;

                Ok(count)
            }
        }
    };
}

macro_rules! impl_paginator {
    ($struct_name:ty) => {
        impl IPaginator for $struct_name {
            fn skip(&self) -> u64 {
                ((self.page - 1) * self.page_size) as u64
            }

            fn limit(&self) -> i64 {
                self.page_size
            }
        }
    };
}
