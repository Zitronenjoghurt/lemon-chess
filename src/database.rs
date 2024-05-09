use crate::entities::{session::Session, user::User};
use dotenvy::dotenv;
use mongodb::{error::Result, options::ClientOptions, Client, Collection};
use std::env;

#[derive(Clone)]
pub struct DB {
    pub client: Client,
    pub session_collection: Collection<Session>,
    pub user_collection: Collection<User>,
}

pub async fn setup() -> Result<DB> {
    dotenv().expect("Failed to load .env");
    let mongo_url = env::var("DB_URL").expect("DB URL not set.");
    let client_options = ClientOptions::parse(mongo_url).await?;
    let client = Client::with_options(client_options)?;
    let db = client.database("LemonChess");

    Ok(DB {
        client,
        session_collection: db.collection("sessions"),
        user_collection: db.collection("users"),
    })
}
