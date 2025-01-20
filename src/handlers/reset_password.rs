use std::path::PathBuf;

use actix_files::NamedFile;
use actix_web::{
    http::{header::ContentType, StatusCode},
    web, HttpResponse,
};
use chrono::{DateTime, Duration, Utc};
use serde_json::json;

use crate::{
    controllers::{reset_password, ses::send_reset_password_email},
    jwt::generate_token_hs256,
    request_types::reset_password::CreateResetPasswordReq,
    state,
};

use super::util::{build_conflict_response, build_error_response};

pub async fn create_reset_password_request(
    req: web::Json<CreateResetPasswordReq>,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let mut con = match app_state.db.acquire().await {
        Ok(c) => c,
        Err(e) => {
            log::error!(
                "An error occurred when tried to acquire a db connection from pool: {}",
                e
            );
            return build_error_response();
        }
    };
    //Check if the email does not have an reset record already created
    match reset_password::get_reset_password_expiration_if_exists(req.email.as_str(), &mut *con)
        .await
    {
        Ok(exists) => {
            if let Some(x) = exists {
                return build_conflict_response(Some(x));
            }
        }
        Err(e) => {
            log::error!("An error occurred when tried to check if the user has already created an reset password record: {}",
                e);
            return build_error_response();
        }
    }

    //Create reset password record on DB
    let rec = match reset_password::create_reset_password(req.email.as_str(), &mut *con).await {
        Ok(r) => r,
        Err(e) => {
            log::error!(
                "An error occurred when tried to create a reset password record in db: {}",
                e
            );
            return build_error_response();
        }
    };

    let exp = match DateTime::parse_from_rfc3339(rec.expires_at.as_str()) {
        Ok(e) => e.with_timezone(&Utc),
        Err(_) => {
            let now = Utc::now();
            let exp = now + Duration::minutes(30);
            exp
        }
    };

    // Create token
    let token = match generate_token_hs256(rec.id.as_str(), exp) {
        Ok(t) => t,
        Err(e) => {
            log::error!("Error when building token for reset password: {}", e);
            return build_error_response();
        }
    };

    //TODO - send password reset link
    let reset_password_link = format!("https://192.168.1.19:3000/password/reset?token={}", token);
    match send_reset_password_email(req.email.as_str(), reset_password_link.as_str()).await {
        Ok(_) => {}
        Err(e) => {
            log::error!(
                "An error occurred when tried to send the reset password link throught email: {}",
                e
            );
            return build_error_response();
        }
    };

    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(
            json!({
            "success": true,
            })
            .to_string(),
        )
}

pub async fn reset_password_form() -> actix_web::Result<NamedFile> {
    let path: PathBuf = "/app/html/reset_password.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

pub async fn get_reset_password_page() {}

pub async fn reset_password() -> HttpResponse {
    //TODO - validate token

    //TODO - update password on user's table

    //TODO - update reset password status: is_reset_password = 1

    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(
            json!({
            "success": true,
            })
            .to_string(),
        )
}
