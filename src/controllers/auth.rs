use crate::model::user::User;
use crate::request_types::auth::CreateUserReq;
use chrono::prelude::{DateTime, Utc};

pub async fn create_session<'a, T>(
    email: &str,
    refresh_token: &str,
    expires_at: DateTime<Utc>,
    con: T,
) -> Result<(), sqlx::error::Error>
where
    T: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    let utc_now: DateTime<Utc> = Utc::now().into();
    let created_at = format!("{}", utc_now.format("%+"));
    let exp_fmt = format!("{}", expires_at.format("%+"));

    let _ = sqlx::query!(
        r#"
        INSERT INTO session (user_email, refresh_token, created_at, expires_at)
        VALUES ($1, $2, $3, $4);
    "#,
        email,
        refresh_token,
        created_at,
        exp_fmt
    )
    .execute(con)
    .await?;

    Ok(())
}

pub async fn get_user<'a, T>(email: &str, con: T) -> Result<User, sqlx::error::Error>
where
    T: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    let res = sqlx::query!(
        r#"SELECT 
            id, email, name,
            password, created_at,
            auth_type, is_email_verified,
            is_premium
           FROM user WHERE email = $1"#,
        email
    )
    .fetch_one(con)
    .await?;

    return Ok(User {
        id: res.id.unwrap(),
        email: res.email,
        name: res.name,
        password: res.password,
        created_at: res.created_at,
        auth_type: res.auth_type,
        is_email_verified: res.is_email_verified as i8,
        is_premium: res.is_premium as i8,
    });
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
    let created_at = format!("{}", utc_now.format("%+"));
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
