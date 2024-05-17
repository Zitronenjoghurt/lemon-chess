use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    error::ApiError,
    game::{
        color::Color,
        position::{Move, Position},
    },
};

#[derive(Deserialize, IntoParams, Default)]
#[into_params(parameter_in = Query)]
pub struct MoveQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub castle_kingside: Option<bool>,
    pub castle_queenside: Option<bool>,
}

impl MoveQuery {
    pub fn convert_to_move(&self) -> Result<(u8, u8, bool, bool), ApiError> {
        if self.castle_kingside == Some(true) {
            return Ok((0, 0, true, false));
        }

        if self.castle_queenside == Some(true) {
            return Ok((0, 0, false, true));
        }

        let from = match self.from.clone() {
            Some(from_str) => Position::try_from(from_str)? as u8,
            None => {
                return Err(ApiError::BadRequest(
                    "Move needs a specified starting cell".to_string(),
                ));
            }
        };

        let to = match self.to.clone() {
            Some(to_str) => Position::try_from(to_str)? as u8,
            None => {
                return Err(ApiError::BadRequest(
                    "Move needs a specified destination cell".to_string(),
                ));
            }
        };

        Ok((from, to, false, false))
    }
}

/// All legal moves for a given color
#[derive(Serialize, Deserialize, ToSchema)]
pub struct LegalMoves {
    /// The color this legal moves are for
    pub color: Color,
    /// If this color is currently the one to move
    pub current_turn: bool,
    /// Move pairs (from, to) chess cells
    pub cells: Vec<Move>,
    /// If the player can castle kingside
    pub castle_kingside: bool,
    /// If the player can castle queenside
    pub castle_queenside: bool,
}
