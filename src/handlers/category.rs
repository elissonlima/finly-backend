use actix_web::{
    http::{header::ContentType, StatusCode},
    web, HttpMessage, HttpRequest, HttpResponse,
};
use serde_json::json;

use crate::{controllers::auth::get_user, handlers::macros, state};

pub async fn upsert_category(
    req: HttpRequest,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let ext = req.extensions();
    let session = macros::get_session!(ext);
    let mut con = macros::get_database_connection!(app_state);
    let user = macros::unwrap_opt_or_unauthorize!(macros::run_async_unwrap!(
        get_user(session.user_email.as_str(), &mut *con,),
        "An error ocurred when tried to get user from database"
    ));

    log::info!("User: {}", user.name);

    HttpResponse::build(StatusCode::CREATED)
        .insert_header(ContentType::json())
        .body(json!({"success": true}).to_string())
}
