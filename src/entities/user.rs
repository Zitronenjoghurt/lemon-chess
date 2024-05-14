use std::collections::HashMap;

use futures::future::try_join_all;
use mongodb::{
    bson::{self, doc},
    options::UpdateOptions,
    Collection,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::ApiError, models::enums::PermissionLevel, utils::time_operations::timestamp_now_nanos,
};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub key: String,
    pub name: String,
    pub display_name: String,
    pub created_stamp: u64,
    pub permission: PermissionLevel,
    #[serde(default)]
    pub last_access_stamp: u64,
    #[serde(default)]
    pub endpoint_usage: HashMap<String, u64>,
    #[serde(default)]
    /// If user was added through a negotiator via discord, this is the discord user id
    pub discord_id: String,
    #[serde(default)]
    pub rate_limiting: HashMap<String, u64>,
}

impl User {
    /// Creates a new discord user
    pub async fn new_from_discord(
        collection: &Collection<User>,
        name: &str,
        display_name: &str,
        id: &str,
    ) -> Result<Self, ApiError> {
        if find_user_by_discord_id(collection, id).await?.is_some() {
            return Err(ApiError::BadRequest(
                "User with the given user id already exists.".to_string(),
            ));
        };

        // Name already exists so it generates a random number added behind the name
        let user_name = if find_user_by_name(collection, &name.to_lowercase())
            .await?
            .is_some()
        {
            let mut rng = rand::thread_rng();
            let random_number = rng.gen_range(100000000..1000000000);
            format!("{}-{}", name, random_number).to_lowercase()
        } else {
            name.to_string().to_lowercase()
        };

        let key = Uuid::new_v4().simple().to_string();
        let current_stamp = timestamp_now_nanos();

        let user = Self {
            key,
            name: user_name.clone(),
            display_name: display_name.to_string(),
            created_stamp: current_stamp,
            permission: PermissionLevel::User,
            last_access_stamp: current_stamp,
            endpoint_usage: HashMap::new(),
            discord_id: id.to_string(),
            rate_limiting: HashMap::new(),
        };

        user.save(collection).await?;

        Ok(user)
    }

    pub async fn rate_limit(
        &mut self,
        collection: &Collection<User>,
        id: &str,
        cooldown_s: u64,
    ) -> Result<(), ApiError> {
        let current_stamp = timestamp_now_nanos();
        let cooldown = cooldown_s * 1000000000;
        match self.rate_limiting.get(id) {
            Some(&last_access_stamp) if current_stamp < last_access_stamp + cooldown => Err(
                ApiError::RateLimited(last_access_stamp + cooldown - current_stamp),
            ),
            _ => {
                self.rate_limiting.insert(id.to_string(), current_stamp);
                self.save(collection).await?;
                Ok(())
            }
        }
    }

    pub async fn save(&self, collection: &Collection<User>) -> Result<(), ApiError> {
        let filter = doc! { "key": &self.key };
        let update = doc! { "$set": bson::to_bson(self)? };
        let options = UpdateOptions::builder().upsert(true).build();

        collection.update_one(filter, update, Some(options)).await?;
        Ok(())
    }

    pub fn use_endpoint(&mut self, method: &str, path: &str) {
        self.last_access_stamp = timestamp_now_nanos();
        *self
            .endpoint_usage
            .entry(format!("{method} {path}"))
            .or_insert(0) += 1;
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

pub async fn find_users_by_keys(
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

pub async fn find_user_by_discord_id(
    collection: &Collection<User>,
    discord_id: &str,
) -> Result<Option<User>, ApiError> {
    let filter = doc! { "discord_id": discord_id };
    let user = collection.find_one(Some(filter), None).await?;
    Ok(user)
}
