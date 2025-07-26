use sqlx::PgPool;

use crate::model::{Account, User};

pub struct AuthController<'a> {
    db_conn: &'a PgPool,
}

impl<'a> AuthController<'a> {
    pub fn new(db_conn: &'a PgPool) -> AuthController<'a> {
        AuthController { db_conn }
    }

    pub async fn create_user(&self, email: &str, name: &str) -> Result<User, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
                INSERT INTO "user" ("email", "name") VALUES ($1, $2) RETURNING id;
            "#,
            email,
            name
        )
        .fetch_one(self.db_conn)
        .await?;

        let user = User {
            id: rec.id,
            email: String::from(email),
            name: String::from(name),
        };

        Ok(user)
    }

    pub async fn create_account(
        &self,
        user_id: i32,
        provider: &str,
        provider_user_id: &str,
    ) -> Result<Account, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
                INSERT INTO "account" ("user_id", "provider", "provider_user_id")
                 VALUES ($1, $2, $3) RETURNING id, account_status;
            "#,
            user_id,
            provider,
            provider_user_id
        )
        .fetch_one(self.db_conn)
        .await?;

        let account = Account {
            id: rec.id,
            user_id,
            account_status: rec.account_status,
            provider: String::from(provider),
            provider_user_id: String::from(provider_user_id),
            access_token: None,
            refresh_token: None,
            id_token: None,
            expires_at: None,
            last_login_at: None,
        };

        Ok(account)
    }
}
