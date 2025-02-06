use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct Session {
    pub id: Uuid,
    pub user_email: String,
    pub created_at: DateTime<Utc>,
    pub refresh_token: String,
    pub refresh_token_expires_at: DateTime<Utc>,
    pub current_access_token: String,
    pub current_access_token_expires_at: DateTime<Utc>,
}

impl Session {
    pub fn build(user_email: &str) -> Self {
        Session {
            id: Uuid::new_v4(),
            user_email: String::from(user_email),
            created_at: Utc::now(),
            refresh_token: String::from(""),
            refresh_token_expires_at: Utc::now(),
            current_access_token: String::from(""),
            current_access_token_expires_at: Utc::now(),
        }
    }

    pub fn is_refresh_token_valid(&self) -> bool {
        if self.refresh_token.is_empty() {
            return false;
        }

        let now = Utc::now();

        if now.gt(&self.refresh_token_expires_at) {
            //Expired
            return false;
        }

        true
    }

    pub fn is_current_access_token_valid(&self) -> bool {
        if self.current_access_token.is_empty() {
            return false;
        }

        let now = Utc::now();

        if now.gt(&self.current_access_token_expires_at) {
            //Expired
            return false;
        }

        true
    }
}
