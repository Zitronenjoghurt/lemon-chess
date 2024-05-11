use crate::entities::session::find_sessions_by_key_with_pagination;
use crate::error::ApiError;
use crate::extractors::authentication::ExtractUser;
use crate::extractors::session_extractor::ExtractSession;
use crate::game::color::Color;
use crate::game::render::render;
use crate::models::move_models::MoveQuery;
use crate::models::query_models::PaginationQuery;
use crate::models::session_models::SessionInfo;
use crate::AppState;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{routing::get, Json, Router};
use image::DynamicImage;
use std::io::Cursor;

/// Retrieve session information.
///
/// This endpoint returns basic session information.
#[utoipa::path(
    get,
    path = "/session",
    responses(
        (status = 200, description = "Session information", body = SessionInfo),
        (status = 400, description = "Missing or invalid session id"),
        (status = 401, description = "Invalid API Key"),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Server error"),
    ),
    params(
        ("session-id" = String, Header, description = "ID of the session"),
      ),
    security(
        ("api_key" = [])
    ),
    tag = "Session"
)]
async fn get_session(
    ExtractUser(user): ExtractUser,
    ExtractSession(session): ExtractSession,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    let info = SessionInfo::from_session(&state, session, user.key).await?;
    Ok(Json(info).into_response())
}

/// Retrieve your current sessions.
///
/// This endpoint returns all your available sessions.
#[utoipa::path(
    get,
    path = "/sessions",
    responses(
        (status = 200, description = "Session information", body = SessionList),
        (status = 401, description = "Invalid API Key"),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Server error"),
    ),
    params(
        PaginationQuery
      ),
    security(
        ("api_key" = [])
    ),
    tag = "Session"
)]
async fn get_sessions(
    ExtractUser(user): ExtractUser,
    State(state): State<AppState>,
    pagination: Query<PaginationQuery>,
) -> Result<Response, ApiError> {
    let (page, page_size) = pagination.retrieve();
    let session_list =
        find_sessions_by_key_with_pagination(&state, user.key, page, page_size).await?;

    Ok(Json(session_list).into_response())
}

/// Retrieve chess board image (10s cooldown).
///
/// This endpoint renders the chess board and returns an image.
#[utoipa::path(
    get,
    path = "/session/render",
    responses(
        (status = 200, description = "Chess board image", content_type = "image/png"),
        (status = 400, description = "Missing or invalid session id"),
        (status = 401, description = "Invalid API Key"),
        (status = 404, description = "Session not found"),
        (status = 429, description = "Rate limited"),
        (status = 500, description = "Server error"),
    ),
    params(
        ("session-id" = String, Header, description = "ID of the session"),
      ),
    security(
        ("api_key" = [])
    ),
    tag = "Session"
)]
async fn get_session_render(
    ExtractUser(mut user): ExtractUser,
    ExtractSession(session): ExtractSession,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    user.rate_limit(&state.database.user_collection, "render", 10)
        .await?;
    let player_color = session
        .get_color_from_key(&user.key)
        .unwrap_or(Color::WHITE);
    let response = match render(session.game_state, player_color).map(DynamicImage::ImageRgba16) {
        Ok(image) => {
            let mut bytes: Vec<u8> = Vec::new();
            let mut cursor = Cursor::new(&mut bytes);
            if image.write_to(&mut cursor, image::ImageFormat::Png).is_ok() {
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "image/png")
                    .body(Body::from(bytes))
                    .unwrap()
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to encode image.".to_string(),
                )
                    .into_response()
            }
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to render image.".to_string(),
        )
            .into_response(),
    };
    Ok(response)
}

/// Retrieve legal session moves.
///
/// This endpoint returns your legal moves in this session.
#[utoipa::path(
    get,
    path = "/session/move",
    responses(
        (status = 200, description = "Legal moves", body = LegalMoves),
        (status = 400, description = "Missing/invalid session id or not a player in this session"),
        (status = 401, description = "Invalid API Key"),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Server error"),
    ),
    params(
        ("session-id" = String, Header, description = "ID of the session"),
      ),
    security(
        ("api_key" = [])
    ),
    tag = "Session"
)]
async fn get_session_move(
    ExtractUser(user): ExtractUser,
    ExtractSession(session): ExtractSession,
) -> Result<Response, ApiError> {
    let color = match session.get_color_from_key(&user.key) {
        Some(color) => color,
        None => {
            return Ok((
                StatusCode::BAD_REQUEST,
                "You're not part of this session.".to_string(),
            )
                .into_response())
        }
    };

    let legal_moves = session.get_legal_moves(color)?;
    Ok(Json(legal_moves).into_response())
}

/// Play a move in a chess session.
///
/// This endpoint allows you to move in a chess session.
#[utoipa::path(
    post,
    path = "/session/move",
    responses(
        (status = 200, description = "Updated session information", body = SessionInfo),
        (status = 400, description = "Missing/invalid session id or unable to play the move"),
        (status = 401, description = "Invalid API Key"),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Server error"),
    ),
    params(
        MoveQuery,
        ("session-id" = String, Header, description = "ID of the session"),
      ),
    security(
        ("api_key" = [])
    ),
    tag = "Session"
)]
async fn post_session_move(
    ExtractUser(user): ExtractUser,
    ExtractSession(mut session): ExtractSession,
    State(state): State<AppState>,
    query: Query<MoveQuery>,
) -> Result<Response, ApiError> {
    session.do_move(&user.key, &query)?;
    session.save(&state.database.session_collection).await?;
    let info = SessionInfo::from_session(&state, session, user.key).await?;
    Ok(Json(info).into_response())
}

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/session", get(get_session))
        .route("/sessions", get(get_sessions))
        .route("/session/render", get(get_session_render))
        .route("/session/move", get(get_session_move))
        .route("/session/move", post(post_session_move))
}
