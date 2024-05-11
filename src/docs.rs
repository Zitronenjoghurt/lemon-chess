use crate::{
    game::color::Color,
    models::{
        move_models::LegalMoves,
        response_models::{MessageResponse, Pagination, UserApiKey},
        room_models::{RoomInfo, RoomList},
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
        resources::room::post_room,
        resources::room::delete_room,
        resources::room::post_room_join,
        resources::room::get_rooms,
        resources::room::get_rooms_public,
        resources::session::get_session,
        resources::session::delete_session,
        resources::session::get_sessions,
        resources::session::get_session_render,
        resources::session::get_session_render_history,
        resources::session::get_session_move,
        resources::session::post_session_move,
        resources::user::post_user_discord,
    ),
    tags(
        (name = "Misc", description = "Miscellaneous endpoints"),
        (name = "User", description = "User endpoints"),
        (name = "Room", description = "Room endpoints"),
        (name = "Session", description = "Session endpoints"),
    ),
    modifiers(&SecurityAddon),
    components(
        schemas(MessageResponse, UserApiKey, SessionInfo, Color, LegalMoves, SessionList, Pagination, RoomInfo, RoomList),
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
