use chrono::{Duration, TimeZone, Utc};
use uuid::Uuid;

use crate::model::reset_password::ResetPassword;

pub async fn create_reset_password<'a, T>(
    email: &str,
    con: T,
) -> Result<ResetPassword, sqlx::error::Error>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let now = Utc::now();
    let exp = now + Duration::minutes(30);
    let id = Uuid::new_v4();

    let rec = ResetPassword {
        id,
        user_email: String::from(email),
        sent_at: now,
        expires_at: exp,
        is_password_reset: false,
    };

    let _ = sqlx::query!(
        r#"
        INSERT INTO reset_password (id, user_email, sent_at, expires_at, is_password_reset)
        VALUES ($1, $2, $3, $4, $5);
    "#,
        rec.id,
        rec.user_email,
        rec.sent_at.naive_utc(),
        rec.expires_at.naive_utc(),
        rec.is_password_reset
    )
    .execute(con)
    .await?;

    Ok(rec)
}

pub async fn get_reset_password_expiration_if_exists<'a, T>(
    email: &str,
    con: T,
) -> Result<Option<String>, sqlx::error::Error>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let res = sqlx::query!(
        r#"
        SELECT MAX(expires_at) AS expires_at FROM reset_password WHERE user_email = $1 AND is_password_reset = false;
    "#,
        email
    )
    .fetch_all(con)
    .await?;

    if res.len() == 0 {
        return Ok(None);
    }

    let rec = res.first().unwrap();
    let exp = Utc.from_utc_datetime(&rec.expires_at.unwrap());

    if Utc::now().gt(&exp) {
        return Ok(None);
    } else {
        return Ok(Some(exp.to_rfc3339()));
    }
}

pub async fn check_reset_password_id<'a, T>(id: &Uuid, con: T) -> Result<bool, sqlx::error::Error>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let res = sqlx::query!(
        r#"
        SELECT expires_at, is_password_reset FROM reset_password WHERE id = $1
    "#,
        id
    )
    .fetch_all(con)
    .await?;

    if res.len() == 0 {
        return Ok(false);
    }

    let row = res.get(0).unwrap();

    //Token used
    if row.is_password_reset {
        return Ok(false);
    }

    let exp = Utc.from_utc_datetime(&row.expires_at);

    if Utc::now().gt(&exp) {
        return Ok(false);
    }

    Ok(true)
}

pub async fn get_reset_password_email_valid_id<'a, T>(
    id: &Uuid,
    con: T,
) -> Result<Option<String>, sqlx::error::Error>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let res = sqlx::query!(
        r#"
        SELECT user_email, expires_at, is_password_reset FROM reset_password WHERE id = $1
    "#,
        id
    )
    .fetch_all(con)
    .await?;

    if res.len() == 0 {
        return Ok(None);
    }

    let row = res.get(0).unwrap();

    //Token used
    if row.is_password_reset {
        return Ok(None);
    }

    let exp = Utc.from_utc_datetime(&row.expires_at);

    if Utc::now().gt(&exp) {
        return Ok(None);
    }

    Ok(Some(String::from(row.user_email.as_str())))
}

pub async fn toggle_reset_password_flag<'a, T>(id: &Uuid, con: T) -> Result<(), sqlx::error::Error>
where
    T: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let _ = sqlx::query!(
        r#"
        UPDATE reset_password SET is_password_reset = true WHERE id = $1;
    "#,
        id
    )
    .execute(con)
    .await?;

    Ok(())
}
