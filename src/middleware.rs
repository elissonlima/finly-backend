use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    http::header::AUTHORIZATION,
    middleware::Next,
    web::Data,
    Error, HttpMessage,
};

use serde_json::json;
use uuid::Uuid;

use crate::{
    controllers::session_mgm::get_session_by_session_id, jwt::verify_token, state::AppState,
};

pub async fn refresh_token_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let auth = req.headers().get(AUTHORIZATION);
    if auth.is_none() {
        return Err(actix_web::error::ErrorUnauthorized(
            json!({"success": false, "message": "unauthorized"}).to_string(),
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
                json!({"success": false, "message": "internal server error"}).to_string(),
            ));
        }
    };

    let claims = match verify_token(token, &app_data.jwt_decoding_key) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Error trying to verify the token: {}", e);
            return Err(actix_web::error::ErrorUnauthorized(
                json!({"success": false, "message": "unauthorized"}).to_string(),
            ));
        }
    };

    req.extensions_mut().insert(claims);

    next.call(req).await
}

pub async fn auth_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let auth = req.headers().get(AUTHORIZATION);
    if auth.is_none() {
        return Err(actix_web::error::ErrorUnauthorized(
            json!({"success": false, "message": "unauthorized"}).to_string(),
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
                json!({"success": false, "message": "internal server error"}).to_string(),
            ));
        }
    };

    let claims = match verify_token(token, &app_data.jwt_decoding_key) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Error trying to verify the token: {}", e);
            return Err(actix_web::error::ErrorUnauthorized(
                json!({"success": false, "message": "unauthorized"}).to_string(),
            ));
        }
    };

    let mut con = match app_data.db.acquire().await {
        Ok(s) => s,
        Err(e) => {
            log::error!(
                "Error trying to acquire connection to session database: {}",
                e
            );
            return Err(actix_web::error::ErrorInternalServerError(
                json!({"success": false, "message": "internal server error"}).to_string(),
            ));
        }
    };

    let session_id = match Uuid::parse_str(claims.sub.as_str()) {
        Ok(s) => s,
        Err(e) => {
            log::error!("it wasn't possible to convert uuid from string {}", e);
            return Err(actix_web::error::ErrorUnauthorized(
                json!({"success": false, "message": "unauthorized"}).to_string(),
            ));
        }
    };

    let session = match get_session_by_session_id(&mut *con, &session_id)
        .await
        .unwrap_or_else(|e| {
            log::error!("Error trying to get session by id; {}", e);
            None
        }) {
        Some(s) => s,
        None => {
            return Err(actix_web::error::ErrorUnauthorized(
                json!({"success": false, "message": "unauthorized"}).to_string(),
            ));
        }
    };

    if !session.is_refresh_token_valid() || !session.is_current_access_token_valid() {
        return Err(actix_web::error::ErrorUnauthorized(
            json!({"success": false, "message": "unauthorized"}).to_string(),
        ));
    }

    req.extensions_mut().insert(session);

    next.call(req).await
    // post-processing
}
