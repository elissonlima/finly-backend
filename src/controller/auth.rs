use sqlx::PgPool;

use crate::model::{Account, AccountProvider, User};

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
                INSERT INTO "user" ("email", "name") VALUES ($1, $2) RETURNING id, is_premium;
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
            is_premium: rec.is_premium,
        };

        Ok(user)
    }

    pub async fn create_account(
        &self,
        user_id: i32,
        provider: AccountProvider,
        provider_user_id: &str,
    ) -> Result<Account, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
                INSERT INTO "account" ("user_id", "provider", "provider_user_id")
                 VALUES ($1, $2, $3) RETURNING id;
            "#,
            user_id,
            provider.clone() as AccountProvider,
            provider_user_id
        )
        .fetch_one(self.db_conn)
        .await?;

        let account = Account {
            id: rec.id,
            user_id,
            provider: provider,
            provider_user_id: String::from(provider_user_id),
            access_token: None,
            access_token_expires_at: None,
            refresh_token: None,
            refresh_token_expires_at: None,
        };

        Ok(account)
    }

    pub async fn get_user(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        let rec = sqlx::query_as!(
            User,
            r#"
                SELECT "id", "email", "name", "is_premium"
                FROM "user"
                WHERE "email" = $1 and is_active = true;
            "#,
            email
        )
        .fetch_one(self.db_conn)
        .await;

        let usr = match rec {
            Ok(u) => u,
            Err(e) => match e {
                sqlx::Error::RowNotFound => {
                    return Ok(None);
                }
                _ => {
                    return Err(e);
                }
            },
        };

        return Ok(Some(usr));
    }

    pub async fn get_account(
        &self,
        user_id: i32,
        provider: AccountProvider,
    ) -> Result<Option<Account>, sqlx::Error> {
        let rec = sqlx::query_as!(
            Account,
            r#"
                SELECT 
                    "id",
                    "user_id",
                    "provider" as "provider!: AccountProvider",
                    "provider_user_id",
                    "access_token",
                    "access_token_expires_at",
                    "refresh_token",
                    "refresh_token_expires_at"
                FROM "account"
                WHERE "user_id" = $1 and "provider" = $2
            "#,
            user_id,
            provider as AccountProvider
        )
        .fetch_one(self.db_conn)
        .await;

        let acc = match rec {
            Ok(a) => a,
            Err(e) => match e {
                sqlx::Error::RowNotFound => {
                    return Ok(None);
                }
                _ => {
                    return Err(e);
                }
            },
        };

        return Ok(Some(acc));
    }

    pub async fn get_accounts(&self, user_id: &i32) -> Result<Vec<Account>, sqlx::Error> {
        let rec = sqlx::query_as!(
            Account,
            r#"
                SELECT 
                    "id",
                    "user_id",
                    "provider" as "provider!: AccountProvider",
                    "provider_user_id",
                    "access_token",
                    "access_token_expires_at",
                    "refresh_token",
                    "refresh_token_expires_at"
                FROM "account"
                WHERE "user_id" = $1
            "#,
            user_id
        )
        .fetch_all(self.db_conn)
        .await?;

        return Ok(rec);
    }

    pub async fn update_account_tokens(&self, account: &Account) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                UPDATE "account"
                SET
                    "access_token" = $1,
                    "access_token_expires_at" = $2,
                    "refresh_token" = $3,
                    "refresh_token_expires_at" = $4
                WHERE
                    "id" = $5;
            "#,
            account.access_token,
            account.access_token_expires_at,
            account.refresh_token,
            account.refresh_token_expires_at,
            account.id
        )
        .execute(self.db_conn)
        .await?;

        return Ok(());
    }
}
