use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::model::reset_password::ResetPassword;

pub async fn create_reset_password<'a, T>(
    email: &str,
    con: T,
) -> Result<ResetPassword, sqlx::error::Error>
where
    T: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    let now = Utc::now();
    let exp = now + Duration::minutes(30);
    let id = Uuid::new_v4().to_string();

    let rec = ResetPassword {
        id,
        user_email: String::from(email),
        sent_at: now.to_rfc3339(),
        expires_at: exp.to_rfc3339(),
        is_password_reset: 0,
    };

    let _ = sqlx::query!(
        r#"
        INSERT INTO reset_password (id, user_email, sent_at, expires_at, is_password_reset)
        VALUES ($1, $2, $3, $4, $5);
    "#,
        rec.id,
        rec.user_email,
        rec.sent_at,
        rec.expires_at,
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
    T: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    let res = sqlx::query!(
        r#"
        SELECT expires_at FROM reset_password WHERE user_email = $1 AND is_password_reset = 0;
    "#,
        email
    )
    .fetch_all(con)
    .await?;

    if res.len() == 0 {
        return Ok(None);
    }

    let mut largest_exp = i64::MIN;
    for row in res.iter() {
        let c = string_datetime_to_epoch(row.expires_at.as_str());
        if c > largest_exp {
            largest_exp = c;
        }
    }

    let exp = match DateTime::from_timestamp(largest_exp, 0) {
        Some(d) => d.with_timezone(&Utc),
        None => {
            return Ok(None);
        }
    };

    if Utc::now().gt(&exp) {
        return Ok(None);
    } else {
        return Ok(Some(exp.to_rfc3339()));
    }
}

fn string_datetime_to_epoch(str_datetime: &str) -> i64 {
    let datetime = match DateTime::parse_from_rfc3339(str_datetime) {
        Ok(d) => d.with_timezone(&Utc).timestamp() as i64,
        Err(_) => 0 as i64,
    };

    datetime
}
