use serde::Serialize;

#[derive(sqlx::FromRow, Clone, Serialize)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub name: String,
    pub is_premium: bool,
}
