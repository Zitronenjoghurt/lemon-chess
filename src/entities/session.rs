use chrono_tz::UTC;
use futures::{stream, StreamExt, TryStreamExt};
use mongodb::{
    bson::{self, doc, oid::ObjectId},
    options::{FindOptions, InsertOneOptions, UpdateOptions},
    Collection,
};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    error::ApiError,
    game::{ai::get_next_move, color::Color, state::GameState},
    models::{
        move_models::{LegalMoves, MoveQuery},
        response_models::Pagination,
        session_models::{SessionInfo, SessionList},
    },
    utils::time_operations::{nanos_to_date, timestamp_now_nanos},
    AppState,
};

use super::user::find_user_by_key;

#[derive(Serialize, Deserialize)]
pub struct Session {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub keys: [String; 2],
    pub created_stamp: u64,
    pub game_state: GameState,
}

impl Session {
    pub fn new(name: String, keys: [String; 2], game_state: GameState) -> Self {
        Self {
            id: None,
            name,
            keys,
            created_stamp: timestamp_now_nanos(),
            game_state,
        }
    }

    pub fn new_ai(name: String, key: String, game_state: GameState) -> Self {
        let mut rng = rand::thread_rng();
        let keys = match rng.gen_bool(0.5) {
            true => ["AI".to_string(), key],
            _ => [key, "AI".to_string()],
        };

        Self {
            id: None,
            name,
            keys,
            created_stamp: timestamp_now_nanos(),
            game_state,
        }
    }

    pub fn do_move(&mut self, key: &str, chess_move: &MoveQuery) -> Result<(), ApiError> {
        if !self.can_move(key.to_string()) {
            return Err(ApiError::BadRequest(
                "You can't move in this game.".to_string(),
            ));
        }

        if !self.is_move_possible(key, chess_move)? {
            return Err(ApiError::BadRequest(
                "The given move is impossible.".to_string(),
            ));
        }

        let color = match self.get_color_from_key(key) {
            Some(color) => color,
            None => {
                return Err(ApiError::BadRequest(
                    "You can't move in this game.".to_string(),
                ))
            }
        };

        let (from, to, kingside_castle, queenside_castle) = chess_move.convert_to_move()?;

        let success = if kingside_castle {
            self.game_state.castle_kingside(color)
        } else if queenside_castle {
            self.game_state.castle_queenside(color)
        } else {
            self.game_state.make_move(from, to)
        }?;

        if !success {
            return Err(ApiError::BadRequest(
                "Unable to play that move.".to_string(),
            ));
        }

        // Do AI move if possible
        self.do_ai_move().map_err(|err| {
            ApiError::ServerError(format!("An error occured while playing the AI: {}", err))
        })?;

        Ok(())
    }

    pub fn do_ai_move(&mut self) -> Result<(), ApiError> {
        if !self.can_move("AI".to_string()) || self.is_finished() {
            return Ok(());
        }

        let next_move = get_next_move(&self.game_state)?;
        self.do_move("AI", &next_move)?;
        Ok(())
    }

    pub fn get_color_from_key(&self, key: &str) -> Option<Color> {
        if key == self.keys[0] {
            Some(Color::WHITE)
        } else if key == self.keys[1] {
            Some(Color::BLACK)
        } else {
            None
        }
    }

    pub fn is_move_possible(&self, key: &str, chess_move: &MoveQuery) -> Result<bool, ApiError> {
        let color = match self.get_color_from_key(key) {
            Some(color) => color,
            None => return Ok(false),
        };

        let (from, to, kingside_castle, queenside_castle) = chess_move.convert_to_move()?;

        if kingside_castle && queenside_castle {
            return Ok(false);
        }

        if kingside_castle && self.game_state.can_castle_kingside[color as usize] {
            return Ok(true);
        }

        if queenside_castle && self.game_state.can_castle_queenside[color as usize] {
            return Ok(true);
        }

        if self.game_state.available_moves[color as usize].has_move(from, to) {
            return Ok(true);
        }

        Ok(false)
    }

    pub fn can_move(&self, key: String) -> bool {
        if self.is_finished() || !self.keys.contains(&key) {
            return false;
        }

        let color = match self.get_color_from_key(&key) {
            Some(color) => color,
            None => return false,
        };

        self.game_state.next_to_move == color as u8
    }

    pub fn get_legal_moves(&self, color: Color) -> Result<LegalMoves, ApiError> {
        let available_moves = &self.game_state.available_moves[color as usize];
        let moves = available_moves.get_moves()?;

        let mut move_pairs: Vec<(String, String)> = Vec::new();
        for m in moves {
            move_pairs.push((m.0.as_str(), m.1.as_str()));
        }

        let legal_moves = LegalMoves {
            color,
            cells: move_pairs,
            current_turn: color as u8 == self.game_state.next_to_move,
            castle_kingside: self.game_state.can_castle_kingside[color as usize],
            castle_queenside: self.game_state.can_castle_queenside[color as usize],
        };

        Ok(legal_moves)
    }

    pub fn is_finished(&self) -> bool {
        self.game_state.winner != 2 || self.game_state.draw
    }

    pub fn resign(&mut self, color: Color) -> Result<(), ApiError> {
        if self.is_finished() {
            return Err(ApiError::BadRequest("Game is already finished".to_string()));
        }

        self.game_state.winner = color.opponent_color() as u8;
        self.game_state.resign = true;
        Ok(())
    }

    pub async fn save(&self, collection: &Collection<Session>) -> Result<(), ApiError> {
        if let Some(id) = &self.id {
            let filter = doc! { "_id": id };
            let update = doc! { "$set": bson::to_bson(self)? };
            let options = UpdateOptions::builder().upsert(true).build();
            collection.update_one(filter, update, Some(options)).await?;
        } else {
            let options = InsertOneOptions::builder().build();
            collection.insert_one(self, Some(options)).await?;
        }
        Ok(())
    }

    pub async fn to_pgn(&self, state: &AppState) -> Result<String, ApiError> {
        let white_player =
            match find_user_by_key(&state.database.user_collection, &self.keys[0]).await? {
                Some(user) => user.display_name,
                None => "Unknown".to_string(),
            };

        let black_player =
            match find_user_by_key(&state.database.user_collection, &self.keys[1]).await? {
                Some(user) => user.display_name,
                None => "Unknown".to_string(),
            };

        let event = format!("LemonChess Online Game: '{}'", self.name);
        let date = nanos_to_date(self.created_stamp, &UTC);

        let result = if !self.is_finished() {
            "*"
        } else if self.game_state.winner != 2 {
            if self.game_state.winner == 0 {
                "1-0"
            } else {
                "0-1"
            }
        } else {
            "1/2-1/2"
        };

        let movetext = self.game_state.get_san();

        let pgn = format!(
            r#"[Event "{}"]
[Site "chess.lemon.industries/docs"]
[Date "{}"]
[White "{}"]
[Black "{}"]
[Result "{}"]
[Annotator "chess.lemon.industries"]
{}"#,
            event, date, white_player, black_player, result, movetext
        );

        Ok(pgn)
    }
}

pub async fn find_session_by_keys(
    collection: &Collection<Session>,
    keys: Vec<String>,
) -> Result<Option<Session>, ApiError> {
    let filter = doc! { "keys": { "$all": keys.clone() }};
    let session = collection.find_one(filter, None).await?;
    Ok(session)
}

pub async fn find_active_session_by_keys(
    collection: &Collection<Session>,
    keys: Vec<String>,
) -> Result<Option<Session>, ApiError> {
    let filter =
        doc! { "keys": { "$all": keys.clone() }, "game_state.winner": 2, "game_state.draw": false};
    let session = collection.find_one(filter, None).await?;
    Ok(session)
}

pub async fn find_sessions_by_key(
    collection: &Collection<Session>,
    key: &str,
) -> Result<Vec<Session>, ApiError> {
    let filter = doc! { "keys": key};
    let cursor = collection.find(filter, None).await?;
    let sessions: Vec<Session> = cursor.try_collect().await?;
    Ok(sessions)
}

pub async fn find_sessions_by_key_and_finished(
    collection: &Collection<Session>,
    key: &str,
    finished: bool,
) -> Result<Vec<Session>, ApiError> {
    let filter = doc! { "keys": key, "finished": finished};
    let cursor = collection.find(filter, None).await?;
    let sessions: Vec<Session> = cursor.try_collect().await?;
    Ok(sessions)
}

pub async fn find_sessions_by_key_with_pagination(
    state: &AppState,
    key: String,
    page: u32,
    page_size: u32,
) -> Result<SessionList, ApiError> {
    let collection = &state.database.session_collection;

    let offset = Pagination::get_offset(page, page_size);
    let find_options = FindOptions::builder()
        .skip(offset as u64)
        .limit(page_size as i64)
        .build();
    let filter = doc! { "keys": &key };

    let total = collection.count_documents(filter.clone(), None).await? as u32;

    let cursor = collection.find(filter, find_options).await?;
    let sessions: Vec<Session> = cursor.try_collect().await?;
    let sessions_info: Vec<SessionInfo> = stream::iter(sessions)
        .then(|session| SessionInfo::from_session(state, session, key.clone()))
        .try_collect()
        .await?;
    let results = sessions_info.len() as u32;

    Ok(SessionList {
        sessions: sessions_info,
        pagination: Pagination::generate(results, total, page, page_size),
    })
}

pub async fn find_session_by_id(
    collection: &Collection<Session>,
    id: &str,
) -> Result<Option<Session>, ApiError> {
    let oid = ObjectId::parse_str(id)?;
    let filter = doc! { "_id": oid };
    let session = collection.find_one(Some(filter), None).await?;
    Ok(session)
}
