use actix_web::{
    HttpMessage, HttpRequest, HttpResponse,
    http::{StatusCode, header::ContentType},
    web,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Write;
use uuid::Uuid;

use crate::{
    app_state, controller::{self, get_access_token, is_google_access_token_expired},
    handler::{error::AppError, macros},
};

#[derive(Serialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub message: String,
}

#[derive(Deserialize)]
pub struct MessageRequest {
    pub msg: String,
}

pub async fn message_recv(
    req: HttpRequest,
    body: web::Json<MessageRequest>,
    app_state: web::Data<app_state::AppState>,
) -> Result<HttpResponse, AppError> {
    let ext = req.extensions();
    let _user_id = macros::get_user_id!(ext);

    let mut service_token = app_state.google_service_token.lock().unwrap();
    log::info!("Current expiration: {}; Is expired? {}", service_token.expires_at, is_google_access_token_expired(&service_token));
    if is_google_access_token_expired(&service_token) {
        let n_token =  get_access_token(&app_state.google_service_key).await?;
        service_token.access_token = n_token.access_token;
        //service_token.expires_at = n_token.expires_in;
        service_token.token_type = n_token.token_type;
    }

    let llm_res = match controller::exec_prompt(
        &body.msg,
        &service_token
    ).await {
        Ok(l) => l.unwrap_or_default(),
        Err(e) => {
            log::error!("An error occurred while trying to reach google API: {}", e);
            return Err(AppError::InternalServerError);
        }
    };

    let mut llm_built_json = String::new();
    for res in llm_res {
        for candidate in res.candidates {
            for part in candidate.content.parts {
                match write!(&mut llm_built_json, "{}", part.text) {
                    Ok(()) => {}
                    Err(_) => return Err(AppError::InternalServerError),
                }
            }
        }
    }

    let msg = MessageResponse {
        id: Uuid::new_v4(),
        message: llm_built_json,
    };

    Ok(HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(json!(msg).to_string()))
}
