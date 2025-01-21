use crate::model::session::Session;

pub async fn create_session<'a, T>(
    con: T,
    session: &Session,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Sqlite>,
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
        session.created_at,
        session.refresh_token,
        session.refresh_token_expires_at,
        session.current_access_token,
        session.current_access_token_expires_at
    )
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
        id: res.id.clone().unwrap(),
        user_email: res.user_email.clone(),
        created_at: res.created_at.clone(),
        refresh_token: res.refresh_token.clone(),
        refresh_token_expires_at: res.refresh_token_expires_at.clone(),
        current_access_token: res.current_access_token.clone(),
        current_access_token_expires_at: res.current_access_token_expires_at.clone(),
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
        id: res.id.clone().unwrap(),
        user_email: res.user_email.clone(),
        created_at: res.created_at.clone(),
        refresh_token: res.refresh_token.clone(),
        refresh_token_expires_at: res.refresh_token_expires_at.clone(),
        current_access_token: res.current_access_token.clone(),
        current_access_token_expires_at: res.current_access_token_expires_at.clone(),
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
    T: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    let _ = sqlx::query!(
        r#"
            UPDATE sessions
                SET current_access_token = $1,
                    current_access_token_expires_at = $2
            WHERE id = $3
        "#,
        session.current_access_token,
        session.current_access_token_expires_at,
        session.id
    )
    .execute(con)
    .await?;

    Ok(())
}
