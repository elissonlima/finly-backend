use actix_web::{http::{header::ContentType, StatusCode}, web, HttpMessage, HttpRequest, HttpResponse};
use serde::Deserialize;
use serde_json::json;

use crate::{app_state, controller::CategoryController, handler::{errors::AppError, macros}};

#[derive(Deserialize)]
pub struct CreateCategoryReq {
    name: String,
    icon_id: i32,
    color: String
}

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

pub async fn list_category_icons(req: HttpRequest, app_state: web::Data<app_state::AppState>) -> Result<HttpResponse, AppError> {
    let ext = req.extensions();
    let user_id = macros::get_user_id!(ext);
    let category_controller = CategoryController::new(&app_state.pool, user_id);

    let icons = macros::unwrap_res_or_app_err_log_err!(
        category_controller.list_category_icons().await,
        AppError::InternalServerError,
        "An Error occurred when tried to fetch categories from database"
    );

    Ok(
    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(
            json!(icons).to_string()
        )
    )
}

pub async fn list_category_colors(
    req: HttpRequest,
    app_state: web::Data<app_state::AppState>
) -> Result<HttpResponse, AppError> {
    let ext = req.extensions();
    let user_id = macros::get_user_id!(ext);
    let category_controller = CategoryController::new(&app_state.pool, user_id);

    let icons = macros::unwrap_res_or_app_err_log_err!(
        category_controller.list_category_colors().await,
        AppError::InternalServerError,
        "An Error occurred when tried to fetch categories from database"
    );

    Ok(
    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(
            json!(icons).to_string()
        )
    )
}

pub async fn create_category(
    req: HttpRequest,
    body: web::Json<CreateCategoryReq>,
    app_state: web::Data<app_state::AppState>
) -> Result<HttpResponse, AppError> {
    let ext = req.extensions();
    let user_id = macros::get_user_id!(ext);
    let category_controller = CategoryController::new(&app_state.pool, user_id);

    // Validate Input Data
    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest);
    }

    let icons =  macros::unwrap_res_or_app_err_log_err!(
        category_controller.list_category_icons_id().await,
        AppError::InternalServerError,
        "An error occurred when tried to get category icons to validate response"
    );

    if !icons.iter().any(|i| *i == body.icon_id) {
        return Err(AppError::BadRequest);
    }

    let colors = macros::unwrap_res_or_app_err_log_err!(
        category_controller.list_category_colors().await,
        AppError::InternalServerError,
        "An error occurred when tried to get category icons to validate response"
    );

    if !colors.iter().any(|s| *s == body.color) {
        return Err(AppError::BadRequest);
    }

    let res = macros::unwrap_res_or_app_err_log_err!(
        category_controller.create(&body.name, &body.icon_id, &body.color).await,
        AppError::InternalServerError,
        "An error occurred when tried to insert new category into database"
    );

    Ok(
        HttpResponse::build(StatusCode::OK)
            .insert_header(ContentType::json())
            .body(
            json!({
                "status": "OK",
                "res": res
            }).to_string()
        )
    )
}