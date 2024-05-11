use futures::TryStreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId},
    options::{InsertOneOptions, UpdateOptions},
    Collection,
};
use serde::{Deserialize, Serialize};

use crate::{
    error::ApiError,
    utils::{random::generate_user_friendly_code, time_operations::timestamp_now_nanos},
};

/// A user will create a room, if another person joins the room will be deleted and a session will be started
#[derive(Serialize, Deserialize)]
pub struct Room {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub key: String,
    pub code: String,
    pub name: String,
    pub created_stamp: u64,
    pub public: bool,
}

impl Room {
    pub async fn new(
        collection: &Collection<Room>,
        key: String,
        name: String,
        public: bool,
    ) -> Result<Self, ApiError> {
        let code = generate_user_friendly_code(6);

        let code_available = room_code_available(collection, &code).await?;
        if !code_available {
            return Err(ApiError::BadRequest("Room code collision".to_string()));
        }

        let room = Self {
            id: None,
            key,
            code,
            name,
            created_stamp: timestamp_now_nanos(),
            public,
        };

        Ok(room)
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

pub async fn find_public_rooms(collection: &Collection<Room>) -> Result<Vec<Room>, ApiError> {
    let filter = doc! { "public": true};
    let cursor = collection.find(filter, None).await?;
    let rooms: Vec<Room> = cursor.try_collect().await?;
    Ok(rooms)
}

pub async fn find_room_by_code(
    collection: &Collection<Room>,
    code: &str,
) -> Result<Option<Room>, ApiError> {
    let filter = doc! { "code": code.to_uppercase() };
    let room = collection.find_one(Some(filter), None).await?;
    Ok(room)
}

pub async fn room_code_available(
    collection: &Collection<Room>,
    code: &str,
) -> Result<bool, ApiError> {
    let room = find_room_by_code(collection, code).await?;
    Ok(room.is_none())
}
