use futures::TryStreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId},
    options::{InsertOneOptions, UpdateOptions},
    Collection,
};
use serde::{Deserialize, Serialize};

use crate::{error::ApiError, game::state::GameState, utils::time_operations::timestamp_now_nanos};

#[derive(Serialize, Deserialize)]
pub struct Session {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub keys: [String; 2],
    pub created_stamp: u64,
    pub game_state: GameState,
}

impl Session {
    pub fn new(name: String, keys: [String; 2], game_state: GameState) -> Self {
        Self {
            id: None,
            name,
            keys,
            created_stamp: timestamp_now_nanos(),
            game_state,
        }
    }

    pub async fn save(&self, collection: &Collection<Session>) -> Result<(), ApiError> {
        if let Some(id) = &self.id {
            let filter = doc! { "_id": id };
            let update = doc! { "$set": bson::to_bson(self)? };
            let options = UpdateOptions::builder().upsert(true).build();
            collection.update_one(filter, update, Some(options)).await?;
        } else {
            let options = InsertOneOptions::builder().build();
            collection.insert_one(self, Some(options)).await?;
        }
        Ok(())
    }
}

pub async fn find_session_by_keys(
    collection: &Collection<Session>,
    keys: Vec<String>,
) -> Result<Option<Session>, ApiError> {
    let filter = doc! { "keys": { "$all": keys.clone() }};
    let session = collection.find_one(filter, None).await?;
    Ok(session)
}

pub async fn find_sessions_by_key(
    collection: &Collection<Session>,
    key: String,
) -> Result<Vec<Session>, ApiError> {
    let filter = doc! { "keys": key};
    let cursor = collection.find(filter, None).await?;
    let sessions: Vec<Session> = cursor.try_collect().await?;
    Ok(sessions)
}

pub async fn find_session_by_id(
    collection: &Collection<Session>,
    id: &str,
) -> Result<Option<Session>, ApiError> {
    let filter = doc! { "_id": id };
    let session = collection.find_one(Some(filter), None).await?;
    Ok(session)
}
