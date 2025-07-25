use actix_web::{
    HttpRequest, HttpResponse,
    http::{StatusCode, header::ContentType},
};
use serde_json::json;

pub async fn auth_login(req: HttpRequest) -> HttpResponse {
    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(
            json!({
                "msg": "Hello, World!"
            })
            .to_string(),
        )
}

