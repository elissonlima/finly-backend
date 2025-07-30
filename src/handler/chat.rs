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
    app_state, controller,
    handler::{errors::AppError, macros},
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
    //let user_id = macros::get_user_id!(ext);

    let llm_res = match controller::exec_prompt(&body.msg).await {
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
