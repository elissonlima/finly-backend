use actix_web::{
    HttpMessage,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    http::header::AUTHORIZATION,
    middleware::Next,
    web::Data,
};
use serde_json::json;

use crate::{app_state::AppState, jwt::verify_token};

pub async fn auth_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let auth = req.headers().get(AUTHORIZATION);
    if auth.is_none() {
        return Err(actix_web::error::ErrorUnauthorized(
            json!({"error":"unauthorized"}).to_string(),
        ));
    }

    let token_h = auth.and_then(|t| t.to_str().ok()).unwrap_or_default();
    let token_s: Vec<&str> = token_h.split_whitespace().collect();
    let mut token = "";

    if token_s.len() > 1 {
        token = token_s[1];
    }

    let app_data = match req.app_data::<Data<AppState>>() {
        Some(e) => e,
        None => {
            log::error!("Error trying to access app state from middleware.");
            return Err(actix_web::error::ErrorInternalServerError(
                json!({"error": "internal server error"}).to_string(),
            ));
        }
    };

    let claims = match verify_token(token, &app_data.jwt_decoding_key) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Error trying to verify the token: {}", e);
            return Err(actix_web::error::ErrorUnauthorized(
                json!({"error":"unauthorized"}).to_string(),
            ));
        }
    };

    let user_id: i32 = match claims.sub.parse() {
        Ok(i) => i,
        Err(e) => {
            log::warn!(
                "it wasn't possible to convert i32 user id from string {}",
                e
            );
            return Err(actix_web::error::ErrorUnauthorized(
                json!({"error": "unauthorized"}).to_string(),
            ));
        }
    };

    req.extensions_mut().insert(user_id);

    next.call(req).await
}
