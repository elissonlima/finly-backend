use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse};

use crate::state;
use crate::request_types::auth::CreateUserReq;
use crate::model::user::User;

pub async fn post_new_user(
    new_user: web::Json<CreateUserReq>,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let mut con = match app_state.db.acquire().await {
        Ok(c) => c,
        Err(e) => {
            log::error!(
                "An error occurred when tried to acquire a db connection from pool: {}",
                e
            );
            return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .insert_header(ContentType::json())
                .body("");
        }
    };

    let u = match User::create_user(&mut *con, new_user.into()).await {
        Ok(u) => u,
        Err(e) => {
            log::error!(
                "An error occurred when tried to insert user on the database: {}",
                e
            );
            return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .insert_header(ContentType::json())
                .body("internal server error");
        }
    };
    log::info!("User created! {}", u.id);

    return HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body("{\"ok\":1");
}
