use crate::model::{user::AuthType, user::User};
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GoogleOauthUserInformation {
    pub aud: String,
    pub azp: String,
    pub email: String,
    pub email_verified: bool,
    pub exp: i64,
    pub family_name: String,
    pub given_name: String,
    pub iat: i64,
    pub iss: String,
    pub name: String,
    pub nonce: String,
    pub picture: String,
    pub sub: String,
}

pub async fn get_user<'a, T>(email: &str, con: T) -> Result<Option<User>, sqlx::error::Error>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let rows = sqlx::query!(
        r#"SELECT 
            id, email, name,
            password, created_at,
            auth_type as "auth_type!: AuthType", 
            google_user_id, is_email_verified,
            is_premium
           FROM "user" WHERE email = $1"#,
        email
    )
    .fetch_all(con)
    .await?;

    if rows.len() == 0 {
        return Ok(None);
    }

    let res = rows.get(0).unwrap();

    return Ok(Some(User {
        id: res.id,
        email: res.email.clone(),
        name: res.name.clone(),
        password: res.password.clone(),
        created_at: Utc.from_utc_datetime(&res.created_at),
        google_user_id: res.google_user_id.clone(),
        auth_type: res.auth_type.clone(),
        is_email_verified: res.is_email_verified,
        is_premium: res.is_premium,
    }));
}

pub async fn check_email_exists<'a, T>(email: &str, con: T) -> Result<bool, sqlx::error::Error>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let res = sqlx::query!(
        r#"
        SELECT COUNT(1) AS c_email FROM "user" WHERE email = $1;
    "#,
        email
    )
    .fetch_one(con)
    .await?;

    match res.c_email {
        Some(c) => {
            if c > 0 {
                return Ok(true);
            }
        }
        None => {
            return Ok(false);
        }
    }

    Ok(false)
}

pub async fn update_password<'a, T>(
    user_email: &str,
    new_password: &str,
    con: T,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let password = bcrypt::hash(new_password, bcrypt::DEFAULT_COST)?;
    let _ = sqlx::query!(
        r#"
        UPDATE "user" SET password = $1 WHERE email = $2;
    "#,
        password,
        user_email
    )
    .execute(con)
    .await?;

    Ok(())
}

pub async fn create_user<'a, T>(con: T, usr: &User) -> Result<User, Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let res = sqlx::query!(
        r#"
            INSERT INTO "user"
                (email, name, password, created_at,
                 auth_type, google_user_id,
                 is_email_verified, is_premium)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id;
        "#,
        usr.email,
        usr.name,
        usr.password,
        usr.created_at.naive_utc(),
        usr.auth_type.clone() as AuthType,
        usr.google_user_id,
        usr.is_email_verified,
        usr.is_premium
    )
    .fetch_one(con)
    .await?;

    let res = User {
        id: res.id,
        email: usr.email.clone(),
        name: usr.name.clone(),
        password: usr.password.clone(),
        created_at: usr.created_at,
        auth_type: usr.auth_type.clone(),
        google_user_id: usr.google_user_id.clone(),
        is_email_verified: usr.is_email_verified,
        is_premium: usr.is_premium,
    };

    Ok(res)
}
