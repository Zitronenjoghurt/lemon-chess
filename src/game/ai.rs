use pleco::{bots::IterativeSearcher, tools::Searcher, Board};

use crate::models::move_models::MoveQuery;

use super::{error::GameError, position::Position, state::GameState};

pub fn get_next_move(state: &GameState) -> Result<MoveQuery, GameError> {
    let fen = state.to_fen();
    let board = Board::from_fen(&fen)?;
    let best_move = IterativeSearcher::best_move(board, 6);

    let move_query = if best_move.is_king_castle() {
        MoveQuery {
            from: None,
            to: None,
            castle_kingside: Some(true),
            castle_queenside: None,
        }
    } else if best_move.is_queen_castle() {
        MoveQuery {
            from: None,
            to: None,
            castle_kingside: None,
            castle_queenside: Some(true),
        }
    } else {
        let from = Position::try_from(best_move.get_src_u8())?;
        let to = Position::try_from(best_move.get_dest_u8())?;
        MoveQuery {
            from: Some(from.as_str()),
            to: Some(to.as_str()),
            castle_kingside: None,
            castle_queenside: None,
        }
    };

    Ok(move_query)
}
