use serde::Deserialize;

use crate::types::UserStatus;

#[derive(Debug, Deserialize)]
pub struct UpdateUserStatusRequest {
    pub status: UserStatus,
}
