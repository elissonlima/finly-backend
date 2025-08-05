use actix_web::{
    HttpMessage, HttpRequest, HttpResponse,
    web
};
use serde::{Deserialize};
use std::fmt::Write;

use crate::{
    app_state, controller::{self, get_access_token, is_google_access_token_expired},
    handler::{errors::AppError, llm_generic_processor::LLMGenericProcessor, macros}, model::LLMGeneratedResponse,
};

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
    let user_id = macros::get_user_id!(ext);

    let mut service_token = app_state.google_service_token.lock().unwrap();
    log::info!("Current expiration: {}; Is expired? {}", service_token.expires_at, is_google_access_token_expired(&service_token));
    if is_google_access_token_expired(&service_token) {
        let n_token =  get_access_token(&app_state.google_service_key).await?;
        service_token.access_token = n_token.access_token;
        service_token.expires_at = n_token.expires_at;
        service_token.token_type = n_token.token_type;
    }

    let llm_res = match controller::exec_prompt(
        &body.msg,
        &service_token,
        controller::PromptCase::UserInputToJson
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

    let llm_response_object: LLMGeneratedResponse = match serde_json::from_str(&llm_built_json){
        Ok(e) => e,
        Err(e) => {
            log::error!("Could not serialize LLM Generated Response: {}", e);
            return Err(AppError::InternalServerError); 
        }
    };

    let llm_processor = LLMGenericProcessor::new(
        &app_state.pool, 
        &llm_response_object, 
        user_id
    );

    return llm_processor.process().await;
}
