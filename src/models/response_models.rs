use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserApiKey {
    pub api_key: String,
}

/// Pagination information for the request results
#[derive(Serialize, Deserialize, ToSchema)]
pub struct Pagination {
    /// The amount of results on the current page
    pub results: u32,
    /// The total amount of results
    pub total: u32,
    /// The current page
    pub page: u32,
    /// The amount of results per page
    pub page_size: u32,
    /// The total amount of pages
    pub pages_total: u32,
    /// The offset applied according to the current page
    pub offset: u32,
}

impl Pagination {
    pub fn get_offset(page: u32, page_size: u32) -> u32 {
        (page - 1) * page_size
    }

    pub fn generate(results: u32, total: u32, page: u32, page_size: u32) -> Self {
        let pages_total = total / page_size;
        let offset = Self::get_offset(page, page_size);

        Self {
            results,
            total,
            page,
            page_size,
            pages_total,
            offset,
        }
    }
}
