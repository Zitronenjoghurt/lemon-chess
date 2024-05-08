use axum::Router;
use std::io;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::game::{
    chess_board::ChessBoard, color::Color, position::Position, render::render, state::GameState,
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
    pub mod render;
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
    let mut state =
        GameState::from_fen("RnRq1rB1/pppQppNp/8/3P1P2/1P1P1R2/2Q3B1/P3P1PP/1k2Kr2 w - - 0 1")
            .unwrap();
    render(state);
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
