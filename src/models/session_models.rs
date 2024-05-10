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
    pub color_to_move: Color,
    pub finished: bool,
    pub winner: Color,
    pub draw: bool,
}

impl From<Session> for SessionInfo {
    fn from(session: Session) -> Self {
        let id = session.id.unwrap_or_default();
        let finished = session.is_finished();

        Self {
            id: id.to_string(),
            name: session.name,
            fen: session.game_state.to_fen(),
            color_to_move: Color::from(session.game_state.next_to_move as usize),
            finished,
            winner: Color::from(session.game_state.winner as usize),
            draw: session.game_state.draw,
        }
    }
}
