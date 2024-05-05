use crate::authentication::ExtractUser;
use crate::{models::response_models::MessageResponse, AppState};
use axum::response::{IntoResponse, Response};
use axum::{routing::get, Json, Router};

/// Ping the API for a response.
///
/// This endpoint returns a simple pong message to indicate that the API is responsive.
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Pong", body = MessageResponse),
        (status = 401, description = "Invalid API Key"),
        (status = 500, description = "Server error"),
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Misc"
)]
async fn get_ping(ExtractUser(_): ExtractUser) -> Response {
    let response = MessageResponse {
        message: "Pong".to_string(),
    };
    Json(response).into_response()
}

pub fn router() -> Router<AppState> {
    Router::<AppState>::new().route("/", get(get_ping))
}
