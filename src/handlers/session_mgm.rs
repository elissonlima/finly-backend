use actix_web::{
    http::{header::ContentType, StatusCode},
    web, HttpMessage, HttpRequest, HttpResponse,
};
use serde_json::json;

use crate::{controllers::session_mgm::delete_session_by_id, handlers::macros, state};

pub async fn ping(req: HttpRequest) -> HttpResponse {
    let ext = req.extensions();
    let session = macros::get_session!(ext);

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
    let session = macros::get_session!(ext);
    let mut con = macros::get_database_connection!(app_state);

    let _ = macros::run_async_unwrap!(
        delete_session_by_id(&mut *con, session.id.as_str()),
        "an error occurred when tried to delete session"
    );

    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(json!({"success": true}).to_string())
}
