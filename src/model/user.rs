#[derive(sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub password: String,
    pub created_at: String,
    pub auth_type: String,
    pub is_email_verified: i8,
    pub is_premium: i8,
}
