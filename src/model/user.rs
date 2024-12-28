use crate::request_types::auth;
use chrono::prelude::{DateTime, Utc};

#[derive(sqlx::FromRow)]
pub struct User {
    pub id: i64,
    email: String,
    name: String,
    password: String,
    created_at: String,
    auth_type: String,
    is_email_verified: i8,
    is_premium: i8,
}

impl User {
    pub async fn create_user<'a, T>(
        con: T,
        req: auth::CreateUserReq,
    ) -> Result<Self, Box<dyn std::error::Error>>
    where
        T: sqlx::Executor<'a, Database = sqlx::Sqlite>,
    {
        let password = bcrypt::hash(req.password, bcrypt::DEFAULT_COST)?;
        let utc_now: DateTime<Utc> = Utc::now().into();
        let created_at = format!("{}", utc_now.format("%+"));
        let auth_type = String::from("USERNAME_PASSWORD");
        let is_email_verified = 0;
        let is_premium = 0;

        let id = sqlx::query(
            r#"
            INSERT INTO user
                (email, name, password, created_at,
                 auth_type, is_email_verified, is_premium)
            VALUES ($1, $2, $3, $4, $5, $6, $7);
        "#,
        )
        .bind(req.email.clone())
        .bind(req.name.clone())
        .bind(password.clone())
        .bind(created_at.clone())
        .bind(auth_type.clone())
        .bind(is_email_verified)
        .bind(is_premium)
        .execute(con)
        .await?
        .last_insert_rowid();

        let res = User {
            id,
            email: req.email,
            name: req.name,
            password,
            created_at,
            auth_type,
            is_email_verified,
            is_premium,
        };

        Ok(res)
    }
}
