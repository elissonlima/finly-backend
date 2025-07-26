use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(sqlx::Type, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[sqlx(type_name = "account_provider")]
#[sqlx(rename_all = "UPPERCASE")]
pub enum AccountProvider {
    Google,
    Apple,
}

#[derive(sqlx::FromRow, Clone, Serialize)]
pub struct Account {
    pub id: i32,
    pub user_id: i32,
    pub provider: AccountProvider,
    pub provider_user_id: String,
    pub access_token: Option<String>,
    pub access_token_expires_at: Option<DateTime<Utc>>,
    pub refresh_token: Option<String>,
    pub refresh_token_expires_at: Option<DateTime<Utc>>,
}
