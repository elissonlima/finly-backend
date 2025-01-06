use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use chrono::{DateTime, Duration, Utc};
use serde_json::json;

use crate::controllers::auth::*;
use crate::jwt::{generate_token, TokenClaims};
use crate::request_types::auth::{CreateUserReq, LoginUserReq};
use crate::state;

pub async fn refresh_token(
    req: HttpRequest,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let ext = req.extensions();
    let claims = match ext.get::<TokenClaims>() {
        Some(c) => c,
        None => {
            log::error!("Could not retrieve claims from request object.");
            return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .insert_header(ContentType::json())
                .body(json!({"success": false, "message": "internal server error"}).to_string());
        }
    };

    let now: DateTime<Utc> = Utc::now().into();
    let access_token_exp = now + Duration::minutes(15);

    let access_token = match generate_token(
        claims.sub.as_str(),
        &app_state.jwt_encoding_key,
        access_token_exp,
    ) {
        Ok(a) => a,
        Err(e) => {
            log::error!("An error occurred while generating access_token: {}", e);
            return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .insert_header(ContentType::json())
                .body(json!({"success": false, "message": "internal server error"}).to_string());
        }
    };

    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(
            json!({
            "success": true,
            "access_token": access_token,
            "access_token_exp": format!("{}", access_token_exp.format("%+")) })
            .to_string(),
        )
}

pub async fn login_user(
    req: web::Json<LoginUserReq>,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let mut con = match app_state.db.acquire().await {
        Ok(c) => c,
        Err(e) => {
            log::error!(
                "An error occurred when tried to acquire a db connection from pool: {}",
                e
            );
            return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .insert_header(ContentType::json())
                .body(json!({"success": false, "message": "internal server error"}).to_string());
        }
    };

    let db_user = match get_user(req.email.as_str(), &mut *con).await {
        Ok(u) => u,
        Err(e) => {
            log::warn!(
                "An error occurred when tried to retrieve user from database: {}",
                e
            );
            return HttpResponse::build(StatusCode::UNAUTHORIZED)
                .insert_header(ContentType::json())
                .body(
                    json!({"success": false, "message": "email or password incorrect"}).to_string(),
                );
        }
    };

    let is_pwd_valid = match bcrypt::verify(req.password.as_str(), db_user.password.as_str()) {
        Ok(i) => i,
        Err(e) => {
            log::error!("An error occurred while verifying user's password: {}", e);
            return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .insert_header(ContentType::json())
                .body(json!({"success": false, "message": "internal server error"}).to_string());
        }
    };

    if !is_pwd_valid {
        return HttpResponse::build(StatusCode::UNAUTHORIZED)
            .insert_header(ContentType::json())
            .body(json!({"success": false, "message": "email or password incorrect"}).to_string());
    }

    let now: DateTime<Utc> = Utc::now().into();
    let access_token_exp = now + Duration::minutes(15);
    let access_token = match generate_token(
        db_user.email.clone().as_str(),
        &app_state.jwt_encoding_key,
        access_token_exp,
    ) {
        Ok(a) => a,
        Err(e) => {
            log::error!("An error occurred while generating access_token: {}", e);
            return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .insert_header(ContentType::json())
                .body(json!({"success": false, "message": "internal server error"}).to_string());
        }
    };

    let refresh_token_exp = now + Duration::days(1);
    let refresh_token = match generate_token(
        db_user.email.clone().as_str(),
        &app_state.jwt_encoding_key,
        refresh_token_exp.clone(),
    ) {
        Ok(a) => a,
        Err(e) => {
            log::error!("An error occurred while generating refresh_token: {}", e);
            return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .insert_header(ContentType::json())
                .body(json!({"success": false, "message": "internal server error"}).to_string());
        }
    };

    return HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(
            json!({
                "user": {
                    "id": db_user.id,
                    "name": db_user.name,
                    "email": db_user.email,
                    "created_at": db_user.created_at,
                    "auth_type": db_user.auth_type,
                    "is_email_verified": db_user.is_email_verified,
                    "is_premium": db_user.is_premium
                },
                "access_token": access_token,
                "access_token_exp": format!("{}", access_token_exp.format("%+")),
                "refresh_token": refresh_token,
                "refresh_token_exp":format!("{}", refresh_token_exp.format("%+")),
                "success": true})
            .to_string(),
        );
}

pub async fn post_new_user(
    new_user: web::Json<CreateUserReq>,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let mut con = match app_state.db.acquire().await {
        Ok(c) => c,
        Err(e) => {
            log::error!(
                "An error occurred when tried to acquire a db connection from pool: {}",
                e
            );
            return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .insert_header(ContentType::json())
                .body(json!({"success": false, "message": "internal server error"}).to_string());
        }
    };

    let email_exists = match check_email_exists(new_user.email.as_str(), &mut *con).await {
        Ok(c) => c,
        Err(e) => {
            log::error!(
                "An error occurred when tried to check if email exists on the db: {}",
                e
            );
            return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .insert_header(ContentType::json())
                .body(json!({"success": false, "message": "internal server error"}).to_string());
        }
    };

    if email_exists {
        return HttpResponse::build(StatusCode::UNAUTHORIZED)
            .insert_header(ContentType::json())
            .body(json!({"success": false, "message": "email already exists"}).to_string());
    }

    let _ = match create_user(&mut *con, new_user.into()).await {
        Ok(u) => u,
        Err(e) => {
            log::error!(
                "An error occurred when tried to insert user on the database: {}",
                e
            );
            return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .insert_header(ContentType::json())
                .body(json!({"success": false, "message": "internal server error"}).to_string());
        }
    };

    return HttpResponse::build(StatusCode::CREATED)
        .insert_header(ContentType::json())
        .body(
            json!({
                "success": true,
                "message": "user created"
            })
            .to_string(),
        );
}
