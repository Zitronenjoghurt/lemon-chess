use crate::error::ApiError;
use crate::extractors::authentication::ExtractUser;
use crate::extractors::session_extractor::ExtractSession;
use crate::models::session_models::SessionInfo;
use crate::AppState;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{routing::get, Json, Router};

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
    ExtractUser(_): ExtractUser,
    ExtractSession(session): ExtractSession,
) -> Response {
    Json(SessionInfo::from(session)).into_response()
}

/// Retrieve legal session moves.
///
/// This endpoint returns your legal moves in this session.
#[utoipa::path(
    get,
    path = "/session/moves",
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
async fn get_session_moves(
    ExtractUser(user): ExtractUser,
    ExtractSession(session): ExtractSession,
) -> Result<Response, ApiError> {
    let color = match session.get_color_from_key(user.key) {
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

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/session", get(get_session))
        .route("/session/moves", get(get_session_moves))
}
