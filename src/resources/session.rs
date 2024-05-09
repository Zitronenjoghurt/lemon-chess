use crate::extractors::authentication::ExtractUser;
use crate::extractors::session_extractor::ExtractSession;
use crate::models::session_models::SessionInfo;
use crate::AppState;
use axum::response::{IntoResponse, Response};
use axum::{routing::get, Json, Router};

/// Ping the API for a response.
///
/// This endpoint returns a simple pong message to indicate that the API is responsive.
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

pub fn router() -> Router<AppState> {
    Router::<AppState>::new().route("/session", get(get_session))
}
