use crate::{
    entities::session::{find_session_by_id, Session},
    error::ApiError,
    AppState,
};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, HeaderName},
};

pub struct ExtractSession(pub Session);

#[async_trait]
impl FromRequestParts<AppState> for ExtractSession {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let session_key_header = HeaderName::from_static("session-id");
        let session_id = parts
            .headers
            .get(&session_key_header)
            .ok_or(ApiError::BadRequest(
                // TODO: This string is allocated even if there is no error!
                // Use lazy evaluation with `ok_or_else`.
                // Look for such patterns in the code.
                "session-id header is missing".to_string(),
            ))?
            .to_str()
            .map_err(|_| ApiError::BadRequest("Invalid session-id format".to_string()))?;

        let session = find_session_by_id(&state.database.session_collection, session_id)
            .await?
            // TODO: See above.
            .ok_or(ApiError::NotFound("Session not found".to_string()))?;

        Ok(ExtractSession(session))
    }
}
