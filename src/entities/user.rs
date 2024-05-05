use futures::future::try_join_all;
use mongodb::{
    bson::{self, doc},
    options::UpdateOptions,
    Collection,
};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

#[derive(Serialize, Deserialize)]
pub struct User {
    pub key: String,
    pub name: String,
    pub display_name: String,
    pub created_stamp: u64,
}

impl User {
    pub async fn save(&self, collection: &Collection<User>) -> Result<(), ApiError> {
        let filter = doc! { "key": &self.key };
        let update = doc! { "$set": bson::to_bson(self)? };
        let options = UpdateOptions::builder().upsert(true).build();

        collection.update_one(filter, update, Some(options)).await?;
        Ok(())
    }
}

pub async fn find_user_by_key(
    collection: &Collection<User>,
    key: &str,
) -> Result<Option<User>, ApiError> {
    let filter = doc! { "key": key };
    let user = collection.find_one(Some(filter), None).await?;
    Ok(user)
}

async fn find_users_by_keys(
    collection: &Collection<User>,
    keys: Vec<&str>,
) -> Result<Vec<Option<User>>, ApiError> {
    let futures = keys
        .into_iter()
        .map(|key| find_user_by_key(collection, key))
        .collect::<Vec<_>>();
    try_join_all(futures).await
}

pub async fn find_user_by_name(
    collection: &Collection<User>,
    name: &str,
) -> Result<Option<User>, ApiError> {
    let filter = doc! { "name": name.to_lowercase() };
    let user = collection.find_one(Some(filter), None).await?;
    Ok(user)
}
