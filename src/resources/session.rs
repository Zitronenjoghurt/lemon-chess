use crate::entities::session::{
    find_active_session_by_keys, find_sessions_by_key_with_pagination, Session,
};
use crate::error::ApiError;
use crate::extractors::authentication::ExtractUser;
use crate::extractors::session_extractor::ExtractSession;
use crate::game::color::Color;
use crate::game::render::{render_board_png, render_history_gif};
use crate::game::state::GameState;
use crate::models::move_models::MoveQuery;
use crate::models::query_models::{PaginationQuery, RenderStyleQuery};
use crate::models::session_models::SessionInfo;
use crate::AppState;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, post};
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
    ExtractUser(user): ExtractUser,
    ExtractSession(mut session): ExtractSession,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    session.do_ai_move()?; // Play AI move if possible, previous errors could have lead to AI not playing
    let info = SessionInfo::from_session(&state, session, user.key).await?;
    Ok(Json(info).into_response())
}

/// Create AI session.
///
/// This endpoint allows you to create an AI session.
#[utoipa::path(
    post,
    path = "/session",
    responses(
        (status = 200, description = "Session successfully created"),
        (status = 400, description = "Can't create session"),
        (status = 401, description = "Invalid API Key"),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Server error"),
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Session"
)]
async fn post_session(
    ExtractUser(user): ExtractUser,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    let session = find_active_session_by_keys(
        &state.database.session_collection,
        [user.key.clone(), "AI".to_string()].to_vec(),
    )
    .await?;

    if session.is_some() {
        return Err(ApiError::BadRequest(
            "You already have an active AI session.".to_string(),
        ));
    }

    let game_state = GameState::new()?;
    let mut new_session = Session::new_ai("AI Game".to_string(), user.key.clone(), game_state);
    new_session.do_ai_move()?; // Does the AI move if the AI goes first
    new_session.save(&state.database.session_collection).await?;
    Ok(Json("AI game started").into_response())
}

/// Retrieve session PGN.
///
/// This endpoint returns the PGN (Portable Game Notation) of the specified session.
#[utoipa::path(
    get,
    path = "/session/pgn",
    responses(
        (status = 200, description = "Session PGN", content_type = "text/plain"),
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
async fn get_session_pgn(
    ExtractUser(_): ExtractUser,
    ExtractSession(session): ExtractSession,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    let pgn = session.to_pgn(&state).await?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain")
        .body(Body::from(pgn))
        .unwrap())
}

/// Resign a session.
///
/// This endpoint allows you to resign a chess game.
#[utoipa::path(
    delete,
    path = "/session",
    responses(
        (status = 200, description = "Session information", body = SessionInfo),
        (status = 400, description = "Missing/invalid session id or can't resign"),
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
async fn delete_session(
    ExtractUser(user): ExtractUser,
    ExtractSession(mut session): ExtractSession,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    let color = match session.get_color_from_key(&user.key) {
        Some(color) => color,
        None => {
            return Err(ApiError::BadRequest(
                "Not a player of this game.".to_string(),
            ))
        }
    };

    session.resign(color)?;
    session.save(&state.database.session_collection).await?;

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
        RenderStyleQuery,
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
    query: Query<RenderStyleQuery>,
) -> Result<Response, ApiError> {
    user.rate_limit(&state.database.user_collection, "render", 10)
        .await?;
    let player_color = session
        .get_color_from_key(&user.key)
        .unwrap_or(Color::WHITE);

    let style = query.retrieve();
    match render_board_png(&session.game_state, player_color, &style) {
        Ok(image_bytes) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "image/png")
            .body(Body::from(image_bytes))
            .unwrap()),
        Err(e) => Err(ApiError::ServerError(format!(
            "Failed to render image: {}",
            e
        ))),
    }
}

/// Retrieve chess board history (30s cooldown).
///
/// This endpoint renders the chess board history and returns a gif.
#[utoipa::path(
    get,
    path = "/session/render/history",
    responses(
        (status = 200, description = "Chess board animated GIF", content_type = "image/gif"),
        (status = 400, description = "Missing/invalid session id or not a player in this session"),
        (status = 401, description = "Invalid API Key"),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Server error"),
    ),
    params(
        RenderStyleQuery,
        ("session-id" = String, Header, description = "ID of the session"),
      ),
    security(
        ("api_key" = [])
    ),
    tag = "Session"
)]
async fn get_session_render_history(
    ExtractUser(mut user): ExtractUser,
    ExtractSession(session): ExtractSession,
    State(state): State<AppState>,
    query: Query<RenderStyleQuery>,
) -> Result<Response, ApiError> {
    user.rate_limit(&state.database.user_collection, "render_gif", 30)
        .await?;

    let player_color = session
        .get_color_from_key(&user.key)
        .unwrap_or(Color::WHITE);

    let style = query.retrieve();
    match render_history_gif(&session.game_state, player_color, &style) {
        Ok(gif_bytes) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "image/gif")
            .body(Body::from(gif_bytes))
            .unwrap()),
        Err(_) => Err(ApiError::ServerError(
            "An error occured while rendering the gif".to_string(),
        )),
    }
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
        .route("/session", post(post_session))
        .route("/session/pgn", get(get_session_pgn))
        .route("/session", delete(delete_session))
        .route("/sessions", get(get_sessions))
        .route("/session/render", get(get_session_render))
        .route("/session/render/history", get(get_session_render_history))
        .route("/session/move", get(get_session_move))
        .route("/session/move", post(post_session_move))
}
