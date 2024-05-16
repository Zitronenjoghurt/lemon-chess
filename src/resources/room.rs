use crate::entities::room::{
    delete_room_by_code, find_public_rooms_with_pagination, find_room_by_code, find_rooms_by_key,
    find_rooms_by_key_with_pagination, Room,
};
use crate::entities::session::{find_sessions_by_key_and_finished, Session};
use crate::error::ApiError;
use crate::extractors::authentication::ExtractUser;
use crate::game::state::GameState;
use crate::models::query_models::{PaginationQuery, RoomCode, RoomCreation};
use crate::models::room_models::RoomInfo;
use crate::AppState;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use rand::Rng;

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
        total_count + 1
    ));
    let public = query.public.unwrap_or(true);

    let room = Room::new(&state.database.room_collection, user.key, name, public).await?;
    room.save(&state.database.room_collection).await?;

    let info = RoomInfo::from_room(&state, room).await?;

    Ok(Json(info).into_response())
}

/// Close a room.
///
/// This endpoint allows you to close one of your multiplayer rooms.
#[utoipa::path(
    delete,
    path = "/room",
    params(RoomCode),
    responses(
        (status = 200, description = "Room successfully deleted"),
        (status = 400, description = "Not your room"),
        (status = 401, description = "Invalid API Key"),
        (status = 404, description = "Room not found"),
        (status = 500, description = "Server error"),
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Room"
)]
async fn delete_room(
    ExtractUser(user): ExtractUser,
    State(state): State<AppState>,
    query: Query<RoomCode>,
) -> Result<Response, ApiError> {
    let room = match find_room_by_code(&state.database.room_collection, &query.code).await? {
        Some(room) => room,
        None => return Err(ApiError::NotFound("Room not found".to_string())),
    };

    if user.key != room.key {
        return Err(ApiError::BadRequest("This is not your room".to_string()));
    }

    delete_room_by_code(&state.database.room_collection, &query.code).await?;
    Ok(Json("Room closed").into_response())
}

/// Join a room.
///
/// This endpoint allows you to join a multiplayer room, which automatically creates a session.
#[utoipa::path(
    post,
    path = "/room/join",
    params(RoomCode),
    responses(
        (status = 200, description = "Game started"),
        (status = 400, description = "Unable to join room"),
        (status = 401, description = "Invalid API Key"),
        (status = 404, description = "Room not found"),
        (status = 429, description = "Rate limited"),
        (status = 500, description = "Server error"),
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Room"
)]
async fn post_room_join(
    ExtractUser(mut user): ExtractUser,
    State(state): State<AppState>,
    query: Query<RoomCode>,
) -> Result<Response, ApiError> {
    // With a 10s delay it takes >400 years to traverse all room codes
    user.rate_limit(&state.database.user_collection, "join_room", 10)
        .await?;

    let room = match find_room_by_code(&state.database.room_collection, &query.code).await? {
        Some(room) => room,
        None => return Err(ApiError::NotFound("Room not found".to_string())),
    };

    if room.key == user.key {
        return Err(ApiError::BadRequest("Can't join your own room".to_string()));
    }

    // Randomly determine color
    let random_bool = tokio::task::block_in_place(|| {
        let mut rng = rand::thread_rng();
        rng.gen_bool(0.5)
    });
    let keys = if random_bool {
        [user.key.clone(), room.key]
    } else {
        [room.key, user.key.clone()]
    };

    let game_state = GameState::new()?;
    let session = Session::new(room.name, keys, game_state);

    delete_room_by_code(&state.database.room_collection, &query.code).await?;
    session.save(&state.database.session_collection).await?;
    Ok(Json("Game started").into_response())
}

/// Retrieve your rooms.
///
/// This endpoint retrieves rooms you have created.
#[utoipa::path(
    get,
    path = "/rooms",
    params(PaginationQuery),
    responses(
        (status = 200, description = "Rooms you have created", body = RoomList),
        (status = 401, description = "Invalid API Key"),
        (status = 500, description = "Server error"),
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Room"
)]
async fn get_rooms(
    ExtractUser(user): ExtractUser,
    State(state): State<AppState>,
    pagination: Query<PaginationQuery>,
) -> Result<Response, ApiError> {
    let (page, page_size) = pagination.retrieve();
    let rooms = find_rooms_by_key_with_pagination(&state, &user.key, page, page_size).await?;
    Ok(Json(rooms).into_response())
}

/// Retrieve public rooms.
///
/// This endpoint retrieves publicly available rooms.
#[utoipa::path(
    get,
    path = "/rooms/public",
    params(PaginationQuery),
    responses(
        (status = 200, description = "Public rooms", body = RoomList),
        (status = 401, description = "Invalid API Key"),
        (status = 500, description = "Server error"),
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Room"
)]
async fn get_rooms_public(
    ExtractUser(_): ExtractUser,
    State(state): State<AppState>,
    pagination: Query<PaginationQuery>,
) -> Result<Response, ApiError> {
    let (page, page_size) = pagination.retrieve();
    let rooms = find_public_rooms_with_pagination(&state, page, page_size).await?;
    Ok(Json(rooms).into_response())
}

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/room", post(post_room))
        .route("/room", delete(delete_room))
        .route("/room/join", post(post_room_join))
        .route("/rooms", get(get_rooms))
        .route("/rooms/public", get(get_rooms_public))
}
