use actix_web::{http::{header::ContentType, StatusCode}, web, HttpMessage, HttpRequest, HttpResponse};
use serde_json::json;

use crate::{app_state, controller::CategoryController, handler::{errors::AppError, macros}};



pub async fn list_categories(
    req: HttpRequest,
    app_state: web::Data<app_state::AppState>,
) -> Result<HttpResponse, AppError> {

    let ext = req.extensions();
    let user_id = macros::get_user_id!(ext);

    let category_controller = CategoryController::new(&app_state.pool, user_id);
    let categories = macros::unwrap_res_or_app_err_log_err!(
        category_controller.list().await,
        AppError::InternalServerError,
        "An Error occurred when tried to fetch categories from database"
    );

    Ok(
        HttpResponse::build(StatusCode::OK)
            .insert_header(ContentType::json())
            .body(
                json!(categories).to_string()
            )
    )
}