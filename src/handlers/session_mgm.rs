use actix_web::{
    http::{header::ContentType, StatusCode},
    web, HttpMessage, HttpRequest, HttpResponse,
};
use serde_json::json;

use crate::{controllers::session_mgm::delete_session_by_id, model::session::Session, state};

use super::util::build_error_response;

pub async fn ping(req: HttpRequest) -> HttpResponse {
    let ext = req.extensions();
    let session = match ext.get::<Session>() {
        Some(c) => c,
        None => {
            log::error!("Could not retrieve sesion from request object.");
            return build_error_response();
        }
    };

    log::info!(
        "[PING RECEIVED] - Session ID: {} - Session Created At: {}",
        session.id,
        session.created_at
    );

    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(json!({"success": true, "message": "pong"}).to_string())
}

pub async fn logout_user(req: HttpRequest, app_state: web::Data<state::AppState>) -> HttpResponse {
    let ext = req.extensions();
    let session = match ext.get::<Session>() {
        Some(c) => c,
        None => {
            log::error!("Could not retrieve sesion from request object.");
            return build_error_response();
        }
    };

    let mut session_db_con = match app_state.db.acquire().await {
        Ok(c) => c,
        Err(e) => {
            log::error!(
                "An error occurred when tried to acquire a connection to session db from pool: {}",
                e
            );
            return build_error_response();
        }
    };

    let _ = match delete_session_by_id(&mut *session_db_con, session.id.as_str()).await {
        Ok(_) => {}
        Err(e) => {
            log::error!("An error occurred when tried to delete session: {}", e);
            return build_error_response();
        }
    };

    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(json!({"success": true}).to_string())
}
