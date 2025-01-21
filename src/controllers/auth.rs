use crate::model::{user::AuthType, user::User};
use crate::request_types::auth::CreateUserReq;
use chrono::prelude::{DateTime, Utc};

pub async fn get_user<'a, T>(email: &str, con: T) -> Result<Option<User>, sqlx::error::Error>
where
    T: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    let rows = sqlx::query!(
        r#"SELECT 
            id, email, name,
            password, created_at,
            auth_type as "auth_type!: AuthType", is_email_verified,
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

pub async fn update_password<'a, T>(
    user_email: &str,
    new_password: &str,
    con: T,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    let password = bcrypt::hash(new_password, bcrypt::DEFAULT_COST)?;
    let _ = sqlx::query!(
        r#"
        UPDATE user SET password = $1 WHERE email = $2;
    "#,
        password,
        user_email
    )
    .execute(con)
    .await?;

    Ok(())
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
    let auth_type = AuthType::UsernamePassword;
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
        password: Some(password),
        created_at,
        auth_type,
        is_email_verified,
        is_premium,
    };

    Ok(res)
}
