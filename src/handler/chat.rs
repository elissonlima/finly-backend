use actix_web::{
    HttpMessage, HttpRequest, HttpResponse,
    web
};
use serde::{Deserialize};
use serde_json::json;
use crate::{
    app_state, controller::{get_access_token, is_google_access_token_expired, LLMCategoryProcessor, LLMController},
    handler::{errors::AppError, macros}, model::ObjectType,
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

    let mut service_token = match app_state.google_service_token.lock(){
        Ok(s) => s,
        Err(e) => {
            log::error!("Could not get Google Service Token from App State: {}", e);
            return Err(AppError::InternalServerError)
        }
    };

    if is_google_access_token_expired(&service_token) {
        let n_token =  get_access_token(&app_state.google_service_key).await?;
        service_token.access_token = n_token.access_token;
        service_token.expires_at = n_token.expires_at;
        service_token.token_type = n_token.token_type;
    }


    let llm_controller = LLMController::new(&service_token);
    let llm_response = match llm_controller.exec_usr_json(&body.msg).await {
            Ok(l) => l,
            Err(e) => {
                log::error!("Could not execute user prompt: {}", e);
                return Err(AppError::InternalServerError)
            }
    };

    
    let res = match llm_response.object {
        ObjectType::Category => {
            let llm_category_processor = LLMCategoryProcessor::new(
                &app_state.pool,
                user_id,
                &service_token,
                &body.msg
            );
            match llm_category_processor.process(&llm_response).await {
                Ok(c) => {
                    c
                },
                Err(e) => {
                    log::error!("Could not process LLM Generated Response: {}", e);
                    return Err(AppError::InternalServerError);
                }
            }
        },
        ObjectType::CreditCard => {
            json!({"message": "Not implemented"})
        },
        ObjectType::Expense => {
            json!({"message": "Not implemented"})
        },
    };

    Ok(HttpResponse::Ok().body(json!({
        "object": &llm_response.object,
        "command": &llm_response.command,
        "res": json!(res).to_string()
    }).to_string()))
}
