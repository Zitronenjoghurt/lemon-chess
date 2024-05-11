use crate::{
    game::color::Color,
    models::{
        move_models::LegalMoves,
        response_models::{MessageResponse, Pagination, UserApiKey},
        session_models::{SessionInfo, SessionList},
    },
    resources,
};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

#[derive(OpenApi)]
#[openapi(
    info(
        title="Lemon Chess",
        description="A chess web service handling multiplayer, sessions and all game logic.\n\nAll available docs: Rapidoc (/docs), Swagger (/swagger) and Redoc (/redoc).\n\nIf you find bugs or have feedback please create an issue here: https://github.com/Zitronenjoghurt/tamagotchi-api/issues"
    ),
    paths(
        resources::ping::get_ping,
        resources::session::get_session,
        resources::session::get_sessions,
        resources::session::get_session_render,
        resources::session::get_session_move,
        resources::session::post_session_move,
        resources::session::post_session_queue,
        resources::user::post_user_discord,
    ),
    tags(
        (name = "Misc", description = "Miscellaneous endpoints"),
        (name = "User", description = "User endpoints"),
        (name = "Session", description = "Session endpoints"),
    ),
    modifiers(&SecurityAddon),
    components(
        schemas(MessageResponse, UserApiKey, SessionInfo, Color, LegalMoves, SessionList, Pagination),
    )
)]
pub struct ApiDoc;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("x-api-key"))),
            )
        }
    }
}
