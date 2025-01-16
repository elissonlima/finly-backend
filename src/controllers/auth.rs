use std::str::FromStr;

use crate::model::{session::Session, user::User};
use crate::request_types::auth::CreateUserReq;
use chrono::prelude::{DateTime, Utc};
use sqlx::Row;

pub async fn get_user<'a, T>(email: &str, con: T) -> Result<Option<User>, sqlx::error::Error>
where
    T: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    let rows = sqlx::query!(
        r#"SELECT 
            id, email, name,
            password, created_at,
            auth_type, is_email_verified,
            is_premium
           FROM user WHERE email = $1"#,
        email
    )
    .fetch_all(con)
    .await?;

    if rows.len() == 0 {
        return Ok(None);
    }

    let res = rows.get(0).unwrap();

    return Ok(Some(User {
        id: res.id.unwrap(),
        email: res.email.clone(),
        name: res.name.clone(),
        password: res.password.clone(),
        created_at: res.created_at.clone(),
        auth_type: res.auth_type.clone(),
        is_email_verified: res.is_email_verified as i8,
        is_premium: res.is_premium as i8,
    }));
}

pub async fn check_email_exists<'a, T>(email: &str, con: T) -> Result<bool, sqlx::error::Error>
where
    T: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    let res = sqlx::query!(
        r#"
        SELECT COUNT(1) AS c_email FROM user WHERE email = $1;
    "#,
        email
    )
    .fetch_one(con)
    .await?;

    if res.c_email > 0 {
        return Ok(true);
    }

    Ok(false)
}

pub async fn create_user<'a, T>(
    con: T,
    req: CreateUserReq,
) -> Result<User, Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    let password = bcrypt::hash(req.password, bcrypt::DEFAULT_COST)?;
    let utc_now: DateTime<Utc> = Utc::now().into();
    let created_at = utc_now.to_rfc3339();
    let auth_type = String::from("USERNAME_PASSWORD");
    let is_email_verified = 0;
    let is_premium = 0;

    let id = sqlx::query!(
        r#"
            INSERT INTO user
                (email, name, password, created_at,
                 auth_type, is_email_verified, is_premium)
            VALUES ($1, $2, $3, $4, $5, $6, $7);
        "#,
        req.email,
        req.name,
        password,
        created_at,
        auth_type,
        is_email_verified,
        is_premium
    )
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

pub async fn create_session<'a, T>(
    con: T,
    session: &Session,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    let _ = sqlx::query(
        r#"
            INSERT INTO session
                (id, user_email, created_at, refresh_token,
                 refresh_token_expires_at,
                 current_access_token, current_access_token_expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7);
        "#,
    )
    .bind(session.id.as_str())
    .bind(session.user_email.as_str())
    .bind(session.created_at.as_str())
    .bind(session.refresh_token.as_str())
    .bind(session.refresh_token_expires_at.as_str())
    .bind(session.current_access_token.as_str())
    .bind(session.current_access_token_expires_at.as_str())
    .execute(con)
    .await?;

    Ok(())
}

pub async fn get_session_by_session_id<'a, T>(
    con: T,
    session_id: &str,
) -> Result<Option<Session>, Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    let rows = sqlx::query(
        r#"
            SELECT
                id, user_email, created_at, refresh_token,
                refresh_token_expires_at, current_access_token,
                current_access_token_expires_at
            FROM session
            WHERE id = $1
        "#,
    )
    .bind(session_id)
    .fetch_all(con)
    .await?;

    if rows.len() == 0 {
        return Ok(None);
    }

    let res = rows.get(0).unwrap();

    let session = Session {
        id: String::from_str(res.try_get("id")?).unwrap(),
        user_email: String::from_str(res.try_get("user_email")?).unwrap(),
        created_at: String::from_str(res.try_get("created_at")?).unwrap(),
        refresh_token: String::from_str(res.try_get("refresh_token")?).unwrap(),
        refresh_token_expires_at: String::from_str(res.try_get("refresh_token_expires_at")?)
            .unwrap(),
        current_access_token: String::from_str(res.try_get("current_access_token")?).unwrap(),
        current_access_token_expires_at: String::from_str(
            res.try_get("current_access_token_expires_at")?,
        )
        .unwrap(),
    };

    Ok(Some(session))
}

pub async fn get_session_by_user_email<'a, T>(
    con: T,
    user_email: &str,
) -> Result<Option<Session>, Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    let rows = sqlx::query(
        r#"
            SELECT
                id, user_email, created_at, refresh_token,
                refresh_token_expires_at, current_access_token,
                current_access_token_expires_at
            FROM session
            WHERE user_email = $1
        "#,
    )
    .bind(user_email)
    .fetch_all(con)
    .await?;

    if rows.len() == 0 {
        return Ok(None);
    }

    let res = rows.get(0).unwrap();

    let session = Session {
        id: String::from_str(res.try_get("id")?).unwrap(),
        user_email: String::from_str(res.try_get("user_email")?).unwrap(),
        created_at: String::from_str(res.try_get("created_at")?).unwrap(),
        refresh_token: String::from_str(res.try_get("refresh_token")?).unwrap(),
        refresh_token_expires_at: String::from_str(res.try_get("refresh_token_expires_at")?)
            .unwrap(),
        current_access_token: String::from_str(res.try_get("current_access_token")?).unwrap(),
        current_access_token_expires_at: String::from_str(
            res.try_get("current_access_token_expires_at")?,
        )
        .unwrap(),
    };

    Ok(Some(session))
}

pub async fn delete_session_by_id<'a, T>(
    con: T,
    session_id: &str,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    let _ = sqlx::query(
        r#"
            DELETE FROM session WHERE session_id = $1;
        "#,
    )
    .bind(session_id)
    .execute(con)
    .await?;

    Ok(())
}

pub async fn update_session<'a, T>(
    con: T,
    session: &Session,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    let _ = sqlx::query(
        r#"
            UPDATE session
                SET current_access_token = $1,
                    current_access_token_expires_at = $2
            WHERE session_id = $3
        "#,
    )
    .bind(session.current_access_token.as_str())
    .bind(session.current_access_token_expires_at.as_str())
    .execute(con)
    .await?;

    Ok(())
}
