use actix_web::{
    HttpResponse,
    http::{StatusCode, header::ContentType},
    web,
};
use reqwest::header::{AUTHORIZATION, HeaderValue};
use serde::Deserialize;
use serde_json::json;

use crate::{
    app_state::{self, AppState},
    controller::AuthController,
    handler::macros,
    model::User,
    route::auth,
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
    let user = macros::unwrap_res_or_error!(
        auth_controller
            .create_user(&user_info.email, &user_info.name)
            .await,
        "An error occurred while trying to create the user on the database"
    );
    let account = macros::unwrap_res_or_error!(
        auth_controller
            .create_account(user.id, "GOOGLE", &user_info.id)
            .await,
        "An error occurred while trying to create the account on the database"
    );

    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(
            json!({
                "msg": "Hello, World!"
            })
            .to_string(),
        )
}
