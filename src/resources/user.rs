use crate::entities::user::{find_user_by_key, User};
use crate::error::ApiError;
use crate::extractors::authentication::ExtractUser;
use crate::models::enums::PermissionLevel;
use crate::models::query_models::DiscordUserCreation;
use crate::models::response_models::UserApiKey;
use crate::AppState;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};

/// Registers a new discord user.
///
/// NEGOTIATOR ONLY! This endpoint registers a discord user from a given name and discord user id.
/// If the api key is given, it tries to link the discord id with the given key.
/// If the key doesn't exist, it will create a new user as usual.
#[utoipa::path(
    post,
    path = "/user/discord",
    params(DiscordUserCreation),
    responses(
        (status = 200, description = "User successfully registered", body = UserApiKey),
        (status = 400, description = "User id already registered"),
        (status = 401, description = "Invalid API Key"),
        (status = 403, description = "No permission to use this endpoint"),
        (status = 500, description = "Server error"),
    ),
    security(
        ("api_key" = [])
    ),
    tag = "User"
)]
async fn post_user_discord(
    ExtractUser(negotiator): ExtractUser,
    State(state): State<AppState>,
    query: Query<DiscordUserCreation>,
) -> Result<Response, ApiError> {
    negotiator
        .permission
        .authenticate(PermissionLevel::Negotiator)?;

    let user = match &query.api_key {
        Some(key) => match find_user_by_key(&state.database.user_collection, key).await? {
            Some(mut user) => {
                user.discord_id = query.id.clone();
                user.save(&state.database.user_collection).await?;
                user
            }
            None => {
                User::new_from_discord(
                    &state.database.user_collection,
                    &query.name,
                    &query.display_name,
                    &query.id,
                )
                .await?
            }
        },
        None => {
            User::new_from_discord(
                &state.database.user_collection,
                &query.name,
                &query.display_name,
                &query.id,
            )
            .await?
        }
    };

    Ok(Json(UserApiKey { api_key: user.key }).into_response())
}

pub fn router() -> Router<AppState> {
    Router::<AppState>::new().route("/user/discord", post(post_user_discord))
}
