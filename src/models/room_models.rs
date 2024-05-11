use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    entities::{room::Room, user::find_user_by_key},
    error::ApiError,
    AppState,
};

use super::response_models::Pagination;

/// Basic room information
#[derive(Serialize, Deserialize, ToSchema)]
pub struct RoomInfo {
    /// The name of the room
    pub name: String,
    /// The name of the user who created this room
    pub user_name: String,
    /// The room code
    pub code: String,
    /// UNIX timestamp in nanoseconds when the room was created
    pub created_stamp: u64,
    /// If the room is publicly visible or not
    pub public: bool,
}

impl RoomInfo {
    pub async fn from_room(state: &AppState, room: Room) -> Result<Self, ApiError> {
        let user = find_user_by_key(&state.database.user_collection, &room.key).await?;

        let user_name = match user {
            Some(user) => user.display_name,
            None => "Unknown".to_string(),
        };

        let info = Self {
            name: room.name,
            user_name,
            code: room.code,
            created_stamp: room.created_stamp,
            public: room.public,
        };

        Ok(info)
    }
}

/// A list of rooms
#[derive(Serialize, Deserialize, ToSchema)]
pub struct RoomList {
    pub rooms: Vec<RoomInfo>,
    pub pagination: Pagination,
}
