use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use serde_json::json;

pub fn build_error_response() -> HttpResponse {
    HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
        .insert_header(ContentType::json())
        .body(json!({"success": false, "message": "internal server error"}).to_string())
}

pub fn build_unauthorized_response(message: Option<String>) -> HttpResponse {
    let res_msg = message.unwrap_or(String::from("unauthorized"));

    HttpResponse::build(StatusCode::UNAUTHORIZED)
        .insert_header(ContentType::json())
        .body(json!({"success": false, "message": res_msg}).to_string())
}

pub fn build_conflict_response(message: Option<String>) -> HttpResponse {
    let res_msg = message.unwrap_or(String::from("conflict"));

    HttpResponse::build(StatusCode::CONFLICT)
        .insert_header(ContentType::json())
        .body(json!({"success": false, "message": res_msg}).to_string())
}

pub fn build_method_not_allowed(message: Option<String>) -> HttpResponse {
    let res_msg = message.unwrap_or(String::from("method not allowed"));
    HttpResponse::build(StatusCode::METHOD_NOT_ALLOWED)
        .insert_header(ContentType::json())
        .body(json!({"success": false, "message": res_msg}).to_string())
}
