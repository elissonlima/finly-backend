use bcrypt::BcryptError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{controllers::auth::GoogleOauthUserInformation, request_types::auth::CreateUserReq};

#[derive(sqlx::Type, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[sqlx(type_name = "auth_type")]
#[sqlx(rename_all = "UPPERCASE")]
pub enum AuthType {
    UsernamePassword,
    Google,
}

#[derive(sqlx::FromRow, Clone)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub name: String,
    pub password: Option<String>,
    pub created_at: DateTime<Utc>,
    pub auth_type: AuthType,
    pub google_user_id: Option<String>,
    pub is_email_verified: bool,
    pub is_premium: bool,
}

impl User {
    pub fn from_google(data: GoogleOauthUserInformation) -> Self {
        User {
            id: -1,
            email: data.email,
            name: data.name,
            password: None,
            created_at: Utc::now(),
            auth_type: AuthType::Google,
            google_user_id: Some(data.sub),
            is_email_verified: data.email_verified,
            is_premium: false,
        }
    }

    pub fn from_signup_request(data: CreateUserReq) -> Result<Self, BcryptError> {
        let password = bcrypt::hash(data.password, bcrypt::DEFAULT_COST)?;
        let utc_now: DateTime<Utc> = Utc::now().into();
        let created_at = utc_now;
        let auth_type = AuthType::UsernamePassword;
        let is_email_verified = true;
        let is_premium = true;

        Ok(User {
            id: -1,
            email: data.email,
            name: data.name,
            password: Some(password),
            created_at,
            auth_type,
            google_user_id: None,
            is_email_verified,
            is_premium,
        })
    }
}
