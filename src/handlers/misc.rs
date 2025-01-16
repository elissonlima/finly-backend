use actix_web::{
    http::{header::ContentType, StatusCode},
    web, HttpMessage, HttpRequest, HttpResponse,
};
use serde_json::json;

use crate::{model::session::Session, state};

use super::util::build_error_response;

pub async fn ping(req: HttpRequest) -> HttpResponse {
    let ext = req.extensions();
    let session = match ext.get::<Session>() {
        Some(c) => c,
        None => {
            return build_error_response(format!("Could not retrieve sesion from request object."));
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
