use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    entities::{session::Session, user::find_user_by_key},
    error::ApiError,
    game::color::Color,
    AppState,
};

use super::response_models::Pagination;

/// Basic session information
#[derive(Serialize, Deserialize, ToSchema)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub white_player: String,
    pub black_player: String,
    /// Forsyth-Edwards Notation of the current game state
    pub fen: String,
    pub color_to_move: Color,
    pub your_turn: bool,
    pub finished: bool,
    pub winner: Color,
    pub draw: bool,
    pub resign: bool,
    pub stalemate: bool,
    pub remis: bool,
}

impl SessionInfo {
    pub async fn from_session(
        state: &AppState,
        session: Session,
        key: String,
    ) -> Result<Self, ApiError> {
        let id = session.id.unwrap_or_default();
        let finished = session.is_finished();
        let your_turn = session.can_move(key);

        let white_player =
            match find_user_by_key(&state.database.user_collection, &session.keys[0]).await? {
                Some(user) => user.display_name,
                None => "Unknown".to_string(),
            };

        let black_player =
            match find_user_by_key(&state.database.user_collection, &session.keys[1]).await? {
                Some(user) => user.display_name,
                None => "Unknown".to_string(),
            };

        let info = Self {
            id: id.to_string(),
            name: session.name,
            white_player,
            black_player,
            fen: session.game_state.to_fen(),
            color_to_move: Color::from(session.game_state.next_to_move as usize),
            your_turn,
            finished,
            winner: Color::from(session.game_state.winner as usize),
            draw: session.game_state.draw,
            resign: session.game_state.resign,
            stalemate: session.game_state.stalemate,
            remis: session.game_state.remis,
        };

        Ok(info)
    }
}

/// Your current available sessions
#[derive(Serialize, Deserialize, ToSchema)]
pub struct SessionList {
    pub sessions: Vec<SessionInfo>,
    pub pagination: Pagination,
}
