use serde::{Deserialize, Serialize};

use crate::error::ApiError;

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, PartialOrd)]
pub enum PermissionLevel {
    #[default]
    User = 0,
    Negotiator = 1,
    Admin = 2,
}

impl PermissionLevel {
    pub fn authenticate(&self, required: Self) -> Result<(), ApiError> {
        if self < &required {
            Err(ApiError::NoPermission("Permission denied.".to_string()))
        } else {
            Ok(())
        }
    }
}
