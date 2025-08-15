use actix_web::{http::{header::ContentType, StatusCode}, web, HttpMessage, HttpRequest, HttpResponse};
use serde::Deserialize;
use serde_json::json;

use crate::{app_state, controller::CreditCardController, handler::{errors::AppError, macros}};

#[derive(Deserialize)]
pub struct CreateCreditCardReq {
    name: String,
    color: String,
    closing_day: i16
}


pub async fn list_credit_card(
    req: HttpRequest,
    app_state: web::Data<app_state::AppState>
) -> Result<HttpResponse, AppError> {

    let ext = req.extensions();
    let user_id = macros::get_user_id!(ext);

    let credit_card_controller = CreditCardController::new(&app_state.pool, &user_id);
    let cards = macros::unwrap_res_or_app_err_log_err!(
        credit_card_controller.list().await,
        AppError::InternalServerError,
        "It wasn't possible to get Credit Card list from database"
    );

    Ok(HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(json!(cards).to_string())
    )
}

pub async fn create_credit_card(
    req: HttpRequest,
    body: web::Json<CreateCreditCardReq>,
    app_state: web::Data<app_state::AppState>
) -> Result<HttpResponse, AppError> {

    let ext = req.extensions();
    let user_id = macros::get_user_id!(ext);

    let credit_card_controller = CreditCardController::new(&app_state.pool, &user_id);

    //Validate Body
    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest);
    }

    if body.color.trim().is_empty() {
        return Err(AppError::BadRequest)
    }

    if body.closing_day < 0 || body.closing_day > 31 {
        return Err(AppError::BadRequest)
    }

    let res = macros::unwrap_res_or_app_err_log_err!(
        credit_card_controller.create(&body.name, &body.color, &body.closing_day).await,
        AppError::InternalServerError,
        "An error happened when tried to create a new Credit Card"
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