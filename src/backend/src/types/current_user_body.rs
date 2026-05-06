use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CurrentUserBody {
    pub id: Uuid,
    pub fullname: String,
    pub email: String,
    pub avatar_url: Option<String>,
}
