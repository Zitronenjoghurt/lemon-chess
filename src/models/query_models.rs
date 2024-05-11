use serde::Deserialize;
use utoipa::IntoParams;

use crate::utils::sanitize;

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DiscordUserCreation {
    /// The discord user id
    pub id: String,
    /// The unique name of the user
    pub name: String,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PaginationQuery {
    /// The results page number
    pub page: Option<u32>,
    /// The maximum amount of results per page, has to be between 1 and 100
    pub page_size: Option<u32>,
}

impl PaginationQuery {
    pub fn sanitize(&self) -> Self {
        let clamped_page_size = self.page_size.map(|size| size.clamp(1, 100));

        PaginationQuery {
            page: self.page,
            page_size: clamped_page_size,
        }
    }
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct RoomCreation {
    /// The name the room should be publicly visible as
    pub name: Option<String>,
    /// If the room is supposed to be public or not | defaults to true
    pub public: Option<bool>,
}

impl RoomCreation {
    pub fn sanitize(&self) -> Self {
        let name = self
            .name
            .as_ref()
            .map(|name| sanitize::limit_string(&sanitize::profanity(name), 64));

        Self {
            name,
            public: self.public,
        }
    }
}
