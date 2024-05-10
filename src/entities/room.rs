use futures::TryStreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId},
    options::{InsertOneOptions, UpdateOptions},
    Collection,
};
use serde::{Deserialize, Serialize};

use crate::{error::ApiError, utils::time_operations::timestamp_now_nanos};

/// A user will create a room, if another person joins the room will be deleted and a session will be started
#[derive(Serialize, Deserialize)]
pub struct Room {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub key: String,
    pub created_stamp: u64,
}

impl Room {
    pub fn new(key: String) -> Self {
        Self {
            id: None,
            key,
            created_stamp: timestamp_now_nanos(),
        }
    }

    pub async fn save(&self, collection: &Collection<Room>) -> Result<(), ApiError> {
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

pub async fn find_rooms_by_key(
    collection: &Collection<Room>,
    key: String,
) -> Result<Vec<Room>, ApiError> {
    let filter = doc! { "key": key};
    let cursor = collection.find(filter, None).await?;
    let rooms: Vec<Room> = cursor.try_collect().await?;
    Ok(rooms)
}
