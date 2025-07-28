use actix_web::{
    HttpMessage, HttpRequest, HttpResponse,
    http::{StatusCode, header::ContentType},
    web,
};
use serde_json::json;

use crate::{app_state, handler::macros};

pub async fn message_recv(
    req: HttpRequest,
    app_state: web::Data<app_state::AppState>,
) -> HttpResponse {
    let ext = req.extensions();
    let user_id = macros::get_user_id!(ext);

    println!("User ID {}", user_id);

    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(json!({"msg": "hello, world!"}).to_string())
}
