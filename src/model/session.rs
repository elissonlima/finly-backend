use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct Session {
    pub id: String,
    pub user_email: String,
    pub created_at: String,
    pub refresh_token: String,
    pub refresh_token_expires_at: String,
    pub current_access_token: String,
    pub current_access_token_expires_at: String,
}

impl Session {
    pub fn build(user_email: &str) -> Self {
        Session {
            id: Uuid::new_v4().to_string(),
            user_email: String::from(user_email),
            created_at: Utc::now().to_rfc3339(),
            refresh_token: String::from(""),
            refresh_token_expires_at: String::from(""),
            current_access_token: String::from(""),
            current_access_token_expires_at: String::from(""),
        }
    }

    pub fn is_refresh_token_valid(&self) -> bool {
        if self.refresh_token.is_empty() {
            return false;
        }

        let exp = match DateTime::parse_from_rfc3339(self.refresh_token_expires_at.as_str()) {
            Ok(e) => e.with_timezone(&Utc),
            Err(err) => {
                log::warn!("[Session Module] Error trying to parse data from string: {err}");
                return false;
            }
        };
        let now = Utc::now();

        if now.gt(&exp) {
            //Expired
            return false;
        }

        true
    }

    pub fn is_current_access_token_valid(&self) -> bool {
        if self.current_access_token.is_empty() {
            return false;
        }

        let exp = match DateTime::parse_from_rfc3339(self.current_access_token_expires_at.as_str())
        {
            Ok(e) => e.with_timezone(&Utc),
            Err(err) => {
                log::warn!("[Session Module] Error trying to parse data from string: {err}");
                return false;
            }
        };
        let now = Utc::now();

        if now.gt(&exp) {
            //Expired
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_refresh_token_validation() {
        //
        let mut session = Session::build("elisson@me");
        assert_eq!(session.is_refresh_token_valid(), false);

        session.refresh_token = String::from("AAAAAAAAAAAAAAAA");
        session.refresh_token_expires_at = String::from("2050-01-17T00:04:28.657290+00:00");
        assert_eq!(session.is_refresh_token_valid(), true);

        session.refresh_token_expires_at = String::from("2025-01-16T00:04:28.657290+00:00");
        assert_eq!(session.is_refresh_token_valid(), false);
    }

    #[test]
    fn test_current_access_token_validation() {
        let mut session = Session::build("elisson@me");
        assert_eq!(session.is_current_access_token_valid(), false);

        session.current_access_token = String::from("BBBBBBBBBBBBBBBBBB");
        session.current_access_token_expires_at = String::from("2050-01-17T00:04:28.657290+00:00");
        assert_eq!(session.is_current_access_token_valid(), true);

        session.current_access_token_expires_at = String::from("2025-01-16T00:04:28.657290+00:00");
        assert_eq!(session.is_current_access_token_valid(), false);
    }
}
