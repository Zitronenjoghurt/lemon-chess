use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DiscordUserCreation {
    /// The discord user id
    pub id: String,
    /// The unique name of the user
    pub name: String,
}
