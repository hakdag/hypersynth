use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize, FromRow)]
pub struct CurrentUserBody {
    pub id: Uuid,
    pub fullname: String,
    pub email: String,
}
