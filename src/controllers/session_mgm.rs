use chrono::{TimeZone, Utc};
use uuid::Uuid;

use crate::model::session::Session;

pub async fn create_session<'a, T>(
    con: T,
    session: &Session,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let _ = sqlx::query!(
        r#"
            INSERT INTO sessions
                (id, user_email, created_at, refresh_token,
                 refresh_token_expires_at,
                 current_access_token, current_access_token_expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7);
        "#,
        session.id,
        session.user_email,
        session.created_at.naive_utc(),
        session.refresh_token,
        session.refresh_token_expires_at.naive_utc(),
        session.current_access_token,
        session.current_access_token_expires_at.naive_utc()
    )
    .execute(con)
    .await?;

    Ok(())
}

pub async fn get_session_by_session_id<'a, T>(
    con: T,
    session_id: &Uuid,
) -> Result<Option<Session>, Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let rows = sqlx::query!(
        r#"
            SELECT
                id, user_email, created_at, refresh_token,
                refresh_token_expires_at, current_access_token,
                current_access_token_expires_at
            FROM sessions
            WHERE id = $1
        "#,
        session_id
    )
    .fetch_all(con)
    .await?;

    if rows.len() == 0 {
        return Ok(None);
    }

    let res = rows.get(0).unwrap();

    let session = Session {
        id: res.id.clone(),
        user_email: res.user_email.clone(),
        created_at: Utc.from_utc_datetime(&res.created_at),
        refresh_token: res.refresh_token.clone(),
        refresh_token_expires_at: Utc.from_utc_datetime(&res.refresh_token_expires_at),
        current_access_token: res.current_access_token.clone(),
        current_access_token_expires_at: Utc
            .from_utc_datetime(&res.current_access_token_expires_at),
    };

    Ok(Some(session))
}

pub async fn get_session_by_user_email<'a, T>(
    con: T,
    user_email: &str,
) -> Result<Option<Session>, Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let rows = sqlx::query!(
        r#"
            SELECT
                id, user_email, created_at, refresh_token,
                refresh_token_expires_at, current_access_token,
                current_access_token_expires_at
            FROM sessions
            WHERE user_email = $1
        "#,
        user_email
    )
    .fetch_all(con)
    .await?;

    if rows.len() == 0 {
        return Ok(None);
    }

    let res = rows.get(0).unwrap();

    let session = Session {
        id: res.id.clone(),
        user_email: res.user_email.clone(),
        created_at: Utc.from_utc_datetime(&res.created_at),
        refresh_token: res.refresh_token.clone(),
        refresh_token_expires_at: Utc.from_utc_datetime(&res.refresh_token_expires_at),
        current_access_token: res.current_access_token.clone(),
        current_access_token_expires_at: Utc
            .from_utc_datetime(&res.current_access_token_expires_at),
    };

    Ok(Some(session))
}

pub async fn delete_session_by_id<'a, T>(
    con: T,
    session_id: &Uuid,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let _ = sqlx::query!(
        r#"
            DELETE FROM sessions WHERE id = $1
        "#,
        session_id
    )
    .execute(con)
    .await?;

    Ok(())
}

pub async fn update_session<'a, T>(
    con: T,
    session: &Session,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let _ = sqlx::query!(
        r#"
            UPDATE sessions
                SET current_access_token = $1,
                    current_access_token_expires_at = $2

            WHERE id = $3
        "#,
        session.current_access_token,
        session.current_access_token_expires_at.naive_utc(),
        session.id
    )
    .execute(con)
    .await?;

    Ok(())
}

pub async fn reset_session<'a, T>(
    con: T,
    session: &Session,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let _ = sqlx::query!(
        r#"
            UPDATE sessions
                SET current_access_token = $1,
                    current_access_token_expires_at = $2,
                    refresh_token = $3,
                    refresh_token_expires_at = $4,
                    id = $5
            WHERE user_email = $6
        "#,
        session.current_access_token,
        session.current_access_token_expires_at.naive_utc(),
        session.refresh_token,
        session.refresh_token_expires_at.naive_utc(),
        session.id,
        session.user_email
    )
    .execute(con)
    .await?;

    Ok(())
}
