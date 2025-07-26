use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(sqlx::FromRow, Clone, Serialize)]
pub struct Account {
    pub id: i32,
    pub user_id: i32,
    pub account_status: String,
    pub provider: String,
    pub provider_user_id: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
}
