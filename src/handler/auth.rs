use actix_web::{
    HttpResponse,
    http::{StatusCode, header::ContentType},
    web,
};
use chrono::{DateTime, Duration, Utc};
use reqwest::header::{AUTHORIZATION, HeaderValue};
use serde::Deserialize;
use serde_json::json;

use crate::{
    app_state::AppState,
    controller::AuthController,
    handler::macros,
    jwt::generate_token,
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
) -> HttpResponse {
    let google_userinfo_api = "https://www.googleapis.com/userinfo/v2/me";

    let header = macros::unwrap_res_or_error!(
        HeaderValue::from_str(&format!("Bearer {}", body.token)),
        "An error occurred while trying to build request header for google sign in"
    );

    let client = reqwest::Client::new();
    let response = macros::unwrap_res_or_bad_request!(
        client
            .get(google_userinfo_api)
            .header(AUTHORIZATION, header)
            .send()
            .await,
        "An error occurred while getting user info from google user api"
    );

    if !response.status().is_success() {
        log::warn!("Got status != success from google user api");
        return HttpResponse::build(StatusCode::BAD_REQUEST)
            .insert_header(ContentType::json())
            .body(json!({"error":"bad request"}).to_string());
    }

    let user_info: UserInfo = macros::unwrap_res_or_error!(
        response.json().await,
        "An error occurred while trying to deserialize the google user api response"
    );

    let auth_controller = AuthController::new(&app_state.pool);

    // Try to get user from database
    let user_q = macros::unwrap_res_or_error!(
        auth_controller.get_user(&user_info.email).await,
        "An error occurred while trying to retrieve the user from the database"
    );

    // Create user if it doesn't exists on database
    let user = match user_q {
        Some(u) => u,
        None => {
            macros::unwrap_res_or_error!(
                auth_controller
                    .create_user(&user_info.email, &user_info.name)
                    .await,
                "An error occurred while trying to create the user on the database"
            )
        }
    };

    let account_q = macros::unwrap_res_or_error!(
        auth_controller
            .get_account(user.id, model::AccountProvider::Google)
            .await,
        "An error occurred while trying to retrieve the acount from the database"
    );

    let account = match account_q {
        Some(a) => a,
        None => {
            macros::unwrap_res_or_error!(
                auth_controller
                    .create_account(user.id, model::AccountProvider::Google, &user_info.id)
                    .await,
                "An error occurred while trying to create the account on the database"
            )
        }
    };

    let now: DateTime<Utc> = Utc::now().into();
    let access_token_exp = now + Duration::minutes(15);
    let access_token = macros::unwrap_res_or_error!(
        generate_token(&user.email, &app_state.jwt_encoding_key, access_token_exp),
        "an error occurred while trying to generate jwt token"
    );
    let refresh_token_exp = now + Duration::days(90);
    let refresh_token = macros::unwrap_res_or_error!(
        generate_token(&user.email, &app_state.jwt_encoding_key, refresh_token_exp),
        "an error occurred while trying to generate jwt token"
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

    macros::unwrap_res_or_error!(
        auth_controller
            .update_account_tokens(&account_updated)
            .await,
        "An error occurred while trying to update the account tokens on database"
    );

    HttpResponse::build(StatusCode::OK)
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
        )
}
