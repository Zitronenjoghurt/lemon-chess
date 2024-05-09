use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::entities::session::Session;

/// Basic session information
#[derive(Serialize, Deserialize, ToSchema)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
}

impl From<Session> for SessionInfo {
    fn from(session: Session) -> Self {
        let id = session.id.unwrap_or_default();
        Self {
            id: id.to_string(),
            name: session.name,
        }
    }
}
