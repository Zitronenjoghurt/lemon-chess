use futures::{stream, StreamExt, TryStreamExt};
use mongodb::{
    bson::{self, doc, oid::ObjectId},
    options::{FindOptions, InsertOneOptions, UpdateOptions},
    Collection,
};
use serde::{Deserialize, Serialize};

use crate::{
    error::ApiError,
    models::{
        response_models::Pagination,
        room_models::{RoomInfo, RoomList},
    },
    utils::{random::generate_user_friendly_code, time_operations::timestamp_now_nanos},
    AppState,
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
    key: &str,
) -> Result<Vec<Room>, ApiError> {
    let filter = doc! { "key": key};
    let cursor = collection.find(filter, None).await?;
    let rooms: Vec<Room> = cursor.try_collect().await?;
    Ok(rooms)
}

pub async fn find_public_rooms_with_pagination(
    state: &AppState,
    page: u32,
    page_size: u32,
) -> Result<RoomList, ApiError> {
    let collection = &state.database.room_collection;

    let offset = Pagination::get_offset(page, page_size);
    let find_options = FindOptions::builder()
        .skip(offset as u64)
        .limit(page_size as i64)
        .build();
    let filter = doc! { "public": true };

    let total = collection.count_documents(filter.clone(), None).await? as u32;

    let cursor = collection.find(filter, find_options).await?;
    let rooms: Vec<Room> = cursor.try_collect().await?;
    let rooms_info: Vec<RoomInfo> = stream::iter(rooms)
        .then(|room| RoomInfo::from_room(state, room))
        .try_collect()
        .await?;
    let results = rooms_info.len() as u32;

    Ok(RoomList {
        rooms: rooms_info,
        pagination: Pagination::generate(results, total, page, page_size),
    })
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

pub async fn delete_room_by_code(
    collection: &Collection<Room>,
    code: &str,
) -> Result<(), ApiError> {
    let filter = doc! { "code": code };
    collection.delete_one(filter, None).await?;
    Ok(())
}
