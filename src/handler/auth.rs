use actix_web::{
    HttpRequest, HttpResponse,
    http::{self, StatusCode, header::ContentType},
    web,
};
use chrono::{DateTime, Duration, Utc};
use reqwest::header::{AUTHORIZATION, HeaderValue};
use serde::Deserialize;
use serde_json::json;

use crate::{
    app_state::AppState,
    controller::AuthController,
    handler::{errors::AppError, macros},
    jwt::{generate_token, verify_token},
    model::{self, Account},
};

#[derive(Deserialize)]
pub struct GoogleSignInReq {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub name: String,
}

pub async fn google_signin(
    body: web::Json<GoogleSignInReq>,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let google_userinfo_api = "https://www.googleapis.com/userinfo/v2/me";

    let header = macros::unwrap_res_and_error!(
        HeaderValue::from_str(&format!("Bearer {}", body.token)),
        AppError::InternalServerError,
        "An error occurred while trying to build request header for google sign in"
    );

    let client = reqwest::Client::new();
    let response = macros::unwrap_res_and_warn!(
        client
            .get(google_userinfo_api)
            .header(AUTHORIZATION, header)
            .send()
            .await,
        AppError::InternalServerError,
        "An error occurred while getting user info from google user api"
    );

    if !response.status().is_success() {
        log::warn!("Got status != success from google user api");
        return Err(AppError::InternalServerError);
    }

    let user_info: UserInfo = macros::unwrap_res_and_error!(
        response.json().await,
        AppError::InternalServerError,
        "An error occurred while trying to deserialize the google user api response"
    );

    let auth_controller = AuthController::new(&app_state.pool);

    // Try to get user from database
    let user_q = macros::unwrap_res_and_error!(
        auth_controller.get_user(&user_info.email).await,
        AppError::InternalServerError,
        "An error occurred while trying to retrieve the user from the database"
    );

    // Create user if it doesn't exists on database
    let user = match user_q {
        Some(u) => u,
        None => {
            macros::unwrap_res_and_error!(
                auth_controller
                    .create_user(&user_info.email, &user_info.name)
                    .await,
                AppError::InternalServerError,
                "An error occurred while trying to create the user on the database"
            )
        }
    };

    let account_q = macros::unwrap_res_and_error!(
        auth_controller
            .get_account(user.id, model::AccountProvider::Google)
            .await,
        AppError::InternalServerError,
        "An error occurred while trying to retrieve the acount from the database"
    );

    let account = match account_q {
        Some(a) => a,
        None => {
            macros::unwrap_res_and_error!(
                auth_controller
                    .create_account(user.id, model::AccountProvider::Google, &user_info.id)
                    .await,
                AppError::InternalServerError,
                "An error occurred while trying to create the account on the database"
            )
        }
    };

    let now: DateTime<Utc> = Utc::now().into();
    let access_token_exp = now + Duration::minutes(15);
    let access_token = macros::unwrap_res_and_error!(
        generate_token(
            &user.id.to_string(),
            &app_state.jwt_encoding_key,
            access_token_exp
        ),
        AppError::InternalServerError,
        "An error occurred while trying to generate jwt token"
    );
    let refresh_token_exp = now + Duration::days(90);
    let refresh_token = macros::unwrap_res_and_error!(
        generate_token(
            &user.id.to_string(),
            &app_state.jwt_encoding_key,
            refresh_token_exp
        ),
        AppError::InternalServerError,
        "An error occurred while trying to generate jwt token"
    );

    let account_updated = Account {
        id: account.id,
        user_id: account.user_id,
        provider: account.provider,
        provider_user_id: account.provider_user_id,
        access_token: Some(access_token),
        access_token_expires_at: Some(access_token_exp),
        refresh_token: Some(refresh_token),
        refresh_token_expires_at: Some(refresh_token_exp),
    };

    macros::unwrap_res_and_error!(
        auth_controller
            .update_account_tokens(&account_updated)
            .await,
        AppError::InternalServerError,
        "An error occurred while trying to update the account tokens on database"
    );

    let res = HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(
            json!({
                "user" : {
                    "id" : user.id,
                    "email": user.email,
                    "name": user.name,
                    "is_premium": user.is_premium
                },
                "account" : {
                    "access_token": account_updated.access_token,
                    "access_token_expires_at": account_updated.access_token_expires_at,
                    "refresh_token" : account_updated.refresh_token,
                    "refresh_token_expires_at": account_updated.refresh_token_expires_at
                }
            })
            .to_string(),
        );

    Ok(res)
}

pub async fn refresh_token(
    req: HttpRequest,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let authorization = match req.headers().get(http::header::AUTHORIZATION) {
        Some(a) => {
            let r = match a.to_str() {
                Ok(s) => s,
                Err(e) => {
                    log::warn!(
                        "Error while trying to convert authorization header into string: {}",
                        e
                    );
                    return Err(AppError::Unauthorized);
                }
            };
            r
        }
        None => {
            return Err(AppError::Unauthorized);
        }
    };

    let token_s: Vec<&str> = authorization.split_whitespace().collect();
    let mut token = "";

    if token_s.len() > 1 {
        token = token_s[1];
    }

    let claims = match verify_token(token, &app_state.jwt_decoding_key) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Error trying to verify the token: {}", e);
            return Err(AppError::Unauthorized);
        }
    };

    let user_id: i32 = match claims.sub.parse() {
        Ok(i) => i,
        Err(e) => {
            log::warn!(
                "it wasn't possible to convert i32 user id from string {}",
                e
            );
            return Err(AppError::Unauthorized);
        }
    };

    println!("USER ID: {}", user_id);

    Ok(HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(json!({"msg": "hello, world!"}).to_string()))
}
