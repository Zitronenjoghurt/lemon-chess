use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{entities::session::Session, game::color::Color};

/// Basic session information
#[derive(Serialize, Deserialize, ToSchema)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    /// Forsyth-Edwards Notation of the current game state
    pub fen: String,
    /// If the session is finished
    pub finished: bool,
    pub color_to_move: Color,
}

impl From<Session> for SessionInfo {
    fn from(session: Session) -> Self {
        let id = session.id.unwrap_or_default();
        Self {
            id: id.to_string(),
            name: session.name,
            fen: session.game_state.to_fen(),
            finished: session.finished,
            color_to_move: Color::from(session.game_state.next_to_move as usize),
        }
    }
}
