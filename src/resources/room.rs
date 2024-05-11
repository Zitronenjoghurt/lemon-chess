use crate::entities::room::{find_rooms_by_key, Room};
use crate::entities::session::find_sessions_by_key_and_finished;
use crate::error::ApiError;
use crate::extractors::authentication::ExtractUser;
use crate::models::query_models::RoomCreation;
use crate::models::room_models::RoomInfo;
use crate::AppState;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};

/// Open a new room.
///
/// This endpoint allows you to open a new multiplayer room.
#[utoipa::path(
    post,
    path = "/room",
    params(RoomCreation),
    responses(
        (status = 200, description = "Room successfully created", body = RoomInfo),
        (status = 400, description = "Session limit reached"),
        (status = 401, description = "Invalid API Key"),
        (status = 403, description = "No permission to use this endpoint"),
        (status = 500, description = "Server error"),
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Room"
)]
async fn post_room(
    ExtractUser(user): ExtractUser,
    State(state): State<AppState>,
    query: Query<RoomCreation>,
) -> Result<Response, ApiError> {
    let rooms = find_rooms_by_key(&state.database.room_collection, &user.key).await?;
    let sessions =
        find_sessions_by_key_and_finished(&state.database.session_collection, &user.key, false)
            .await?;

    let unfinished_count = rooms.len() + sessions.len();
    if unfinished_count > 10 {
        return Err(ApiError::BadRequest(
            "Maximum session limit of 10 reached.".to_string(),
        ));
    }

    let finished_sessions =
        find_sessions_by_key_and_finished(&state.database.session_collection, &user.key, true)
            .await?;

    let total_count = unfinished_count + finished_sessions.len();

    let query = query.sanitize();
    let name = query.name.unwrap_or(format!(
        "{}'s GAME #{}",
        user.display_name.to_uppercase(),
        total_count
    ));
    let public = query.public.unwrap_or(true);

    let room = Room::new(&state.database.room_collection, user.key, name, public).await?;
    room.save(&state.database.room_collection).await?;

    let info = RoomInfo::from_room(&state, room).await?;

    Ok(Json(info).into_response())
}

pub fn router() -> Router<AppState> {
    Router::<AppState>::new().route("/room", post(post_room))
}
