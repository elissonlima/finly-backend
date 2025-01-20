use serde::{Deserialize, Serialize};

#[derive(sqlx::Type, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuthType {
    UsernamePassword,
    Google,
}

#[derive(sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub password: Option<String>,
    pub created_at: String,
    pub auth_type: AuthType,
    pub is_email_verified: i8,
    pub is_premium: i8,
}
