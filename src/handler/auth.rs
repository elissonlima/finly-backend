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
    handler::{error::AppError, macros},
    jwt::{generate_token, verify_token},
    model::{self, Account},
};

#[derive(Deserialize)]
pub struct GoogleSignInReq {
    pub token: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
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

    let header = macros::unwrap_res_or_app_err_log_err!(
        HeaderValue::from_str(&format!("Bearer {}", body.token)),
        AppError::InternalServerError,
        "An error occurred while trying to build request header for google sign in"
    );

    let client = reqwest::Client::new();
    let response = macros::unwrap_res_or_app_err_log_warn!(
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

    let user_info: UserInfo = macros::unwrap_res_or_app_err_log_err!(
        response.json().await,
        AppError::InternalServerError,
        "An error occurred while trying to deserialize the google user api response"
    );

    let auth_controller = AuthController::new(&app_state.pool);

    // Try to get user from database
    let user_q = macros::unwrap_res_or_app_err_log_err!(
        auth_controller.get_user(&user_info.email).await,
        AppError::InternalServerError,
        "An error occurred while trying to retrieve the user from the database"
    );

    // Create user if it doesn't exists on database
    let user = match user_q {
        Some(u) => u,
        None => {
            macros::unwrap_res_or_app_err_log_err!(
                auth_controller
                    .create_user(&user_info.email, &user_info.name)
                    .await,
                AppError::InternalServerError,
                "An error occurred while trying to create the user on the database"
            )
        }
    };

    let account_q = macros::unwrap_res_or_app_err_log_err!(
        auth_controller
            .get_account(user.id, model::AccountProvider::Google)
            .await,
        AppError::InternalServerError,
        "An error occurred while trying to retrieve the acount from the database"
    );

    let account = match account_q {
        Some(a) => a,
        None => {
            macros::unwrap_res_or_app_err_log_err!(
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
    let access_token = macros::unwrap_res_or_app_err_log_err!(
        generate_token(
            &user.id.to_string(),
            &app_state.jwt_encoding_key,
            access_token_exp
        ),
        AppError::InternalServerError,
        "An error occurred while trying to generate jwt token"
    );
    let refresh_token_exp = now + Duration::days(90);
    let refresh_token = macros::unwrap_res_or_app_err_log_err!(
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

    macros::unwrap_res_or_app_err_log_err!(
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
    let authorization_header_val = macros::unwrap_opt_or_app_err_log_err!(
        req.headers().get(http::header::AUTHORIZATION),
        AppError::Unauthorized,
        "Missing AUTHORIZATION header in refresh_token request"
    );

    let authorization = macros::unwrap_res_or_app_err_log_warn!(
        authorization_header_val.to_str(),
        AppError::InternalServerError,
        "Error while trying to convert authorization header into string"
    );

    let token_s: Vec<&str> = authorization.split_whitespace().collect();
    let mut token = "";

    if token_s.len() > 1 {
        token = token_s[1];
    }

    let claims = macros::unwrap_res_or_app_err_log_warn!(
        verify_token(&token, &app_state.jwt_decoding_key),
        AppError::Unauthorized,
        "Error trying to verify the token"
    );

    let user_id: i32 = macros::unwrap_res_or_app_err_log_err!(
        claims.sub.parse(),
        AppError::InternalServerError,
        "It wasn't possible to convert i32 user id from string"
    );

    let auth_controller = AuthController::new(&app_state.pool);

    let accounts = macros::unwrap_res_or_app_err_log_err!(
        auth_controller.get_accounts(&user_id).await,
        AppError::InternalServerError,
        "Error getting list of accounts"
    );

    let accounts_refresh_tok_filter: Vec<Account> = accounts
        .into_iter()
        .filter(|acc| acc.refresh_token.clone().unwrap_or_default() == token)
        .collect();

    if accounts_refresh_tok_filter.len() != 1 {
        log::warn!(
            "User has {} account(s) with the refresh_token value filtered",
            accounts_refresh_tok_filter.len()
        );
        return Err(AppError::Unauthorized);
    }

    let account = accounts_refresh_tok_filter[0].clone();

    let now: DateTime<Utc> = Utc::now().into();
    let access_token_exp = now + Duration::minutes(15);
    let access_token = macros::unwrap_res_or_app_err_log_err!(
        generate_token(
            &user_id.to_string(),
            &app_state.jwt_encoding_key,
            access_token_exp
        ),
        AppError::InternalServerError,
        "An error occurred while trying to generate jwt token"
    );

    macros::unwrap_res_or_app_err_log_err!(
        auth_controller
            .update_account_tokens(&Account {
                id: account.id,
                user_id: account.user_id,
                provider: account.provider,
                provider_user_id: account.provider_user_id,
                access_token: Some(access_token.clone()),
                access_token_expires_at: Some(access_token_exp),
                refresh_token: account.refresh_token,
                refresh_token_expires_at: account.refresh_token_expires_at,
            })
            .await,
        AppError::InternalServerError,
        "An error occurred while trying to update account on database"
    );

    Ok(HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(
            json!({"access_token": access_token, "access_token_expires_at":access_token_exp})
                .to_string(),
        ))
}
