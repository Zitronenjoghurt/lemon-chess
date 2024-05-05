use crate::{
    entities::user::{find_user_by_key, User},
    error::ApiError,
    AppState,
};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, HeaderName},
};

pub struct ExtractUser(pub User);

#[async_trait]
impl FromRequestParts<AppState> for ExtractUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let api_key_header = HeaderName::from_static("x-api-key");
        let api_key = parts
            .headers
            .get(&api_key_header)
            .ok_or(ApiError::AuthorizationError(
                "API key header is missing, check /docs for more information".to_string(),
            ))?
            .to_str()
            .map_err(|_| {
                ApiError::AuthorizationError(
                    "Invalid API key format, check /docs for more information".to_string(),
                )
            })?;

        let user = find_user_by_key(&state.database.user_collection, api_key)
            .await?
            .ok_or(ApiError::AuthorizationError(
                "Invalid API key, check /docs for more information".to_string(),
            ))?;

        user.save(&state.database.user_collection).await?;

        Ok(ExtractUser(user))
    }
}
