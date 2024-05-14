use serde::Deserialize;
use utoipa::IntoParams;

use crate::{game::render::RenderStyle, utils::sanitize};

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DiscordUserCreation {
    /// The discord user id
    pub id: String,
    /// The unique name of the user
    pub name: String,
    /// The name other people will see
    pub display_name: String,
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

    pub fn retrieve(&self) -> (u32, u32) {
        let query = self.sanitize();
        (query.page.unwrap_or(1), query.page_size.unwrap_or(10))
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

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct RoomCode {
    /// The code of the room
    pub code: String,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct RenderStyleQuery {
    /// The style that should be used for rendering
    pub style: Option<RenderStyle>,
}

impl RenderStyleQuery {
    pub fn retrieve(&self) -> RenderStyle {
        self.style.unwrap_or(RenderStyle::MODERN)
    }
}
