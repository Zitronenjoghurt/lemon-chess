use axum::Router;
use std::io;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

mod database;
mod docs;
pub mod error;

pub mod entities {
    pub mod room;
    pub mod session;
    pub mod user;
}

pub mod extractors {
    pub mod authentication;
    pub mod session_extractor;
}

pub mod game {
    pub mod ai;
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
    pub mod enums;
    pub mod move_models;
    pub mod query_models;
    pub mod response_models;
    pub mod room_models;
    pub mod session_models;
}

pub mod resources {
    pub mod ping;
    pub mod room;
    pub mod session;
    pub mod user;
}

pub mod utils {
    pub mod random;
    pub mod sanitize;
    pub mod time_operations;
}

#[derive(Clone)]
pub struct AppState {
    database: database::DB,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let db = database::setup().await.expect("Failed to set up MongoDB.");

    let app_state = AppState { database: db };

    let app = Router::<AppState>::new()
        .nest("/", resources::ping::router())
        .nest("/", resources::room::router())
        .nest("/", resources::session::router())
        .nest("/", resources::user::router())
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", docs::ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", docs::ApiDoc::openapi()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/docs"))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await
}
