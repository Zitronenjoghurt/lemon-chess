use axum::Router;
use std::io;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::game::{
    bit_board::BitBoard, chess_board::ChessBoard, color::Color, piece::Piece, position::Position,
    state::GameState,
};

pub mod authentication;
mod database;
mod docs;
pub mod error;

pub mod entities {
    pub mod user;
}

pub mod game {
    pub mod bit_board;
    pub mod chess_board;
    pub mod color;
    pub mod error;
    pub mod piece;
    pub mod position;
    pub mod state;
}

pub mod models {
    pub mod response_models;
}

pub mod resources {
    pub mod ping;
}

#[derive(Clone)]
pub struct AppState {
    database: database::DB,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // Testing legal move generation
    //let state = GameState::new().unwrap();
    //let moves = state.get_legal_moves(Color::BLACK);
    let chess_board = ChessBoard::default();
    let test: Vec<String> = Piece::get_king_threat_masks(5, chess_board.colors[0])
        .iter()
        .map(|board| board.to_string())
        .collect();

    let db = database::setup().await.expect("Failed to set up MongoDB.");

    let app_state = AppState { database: db };

    let app = Router::<AppState>::new()
        .nest("/", resources::ping::router())
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", docs::ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", docs::ApiDoc::openapi()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/docs"))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await
}
