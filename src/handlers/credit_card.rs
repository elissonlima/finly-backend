use actix_web::{
    http::{header::ContentType, StatusCode},
    web, HttpMessage, HttpRequest, HttpResponse,
};
use chrono::DateTime;
use serde_json::json;
use sqlx::Acquire;
use uuid::Uuid;

use crate::{
    controllers::{self, auth::get_user},
    handlers::{macros, util::build_status_code_for_multiple_input},
    model::credit_card::CreditCard,
    request_types::credit_card::{
        CreateBillAtDateReq, CreateBillRes, DeleteCreditCardReq, ListCreditCardBillsReq,
        ListCreditCardsRes, UpsertCreditCardReq,
    },
    state,
};

pub async fn upsert_credit_card(
    req: HttpRequest,
    body: web::Json<Vec<UpsertCreditCardReq>>,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let ext = req.extensions();
    let session = macros::get_session!(ext);
    let mut con = macros::get_database_connection!(app_state);
    let user = macros::unwrap_opt_or_unauthorize!(macros::run_async_unwrap!(
        get_user(session.user_email.as_str(), &mut *con,),
        "An error ocurred when tried to get user from database"
    ));

    let mut tx = macros::begin_transaction!(con);
    let mut not_created: Vec<Uuid> = Vec::new();

    for card_req in body.iter() {
        let card = CreditCard {
            id: card_req.id,
            user_id: user.id,
            name: card_req.name.clone(),
            icon_name: card_req.icon_name.clone(),
            limit_value: card_req.limit_value,
            closing_day: card_req.closing_day,
        };

        if card.name.is_empty()
            || card.icon_name.is_empty()
            || card.limit_value < 0
            || card.closing_day < 0
            || card.closing_day > 31
        {
            not_created.push(card.id.clone());
            continue;
        }
        macros::run_async_or!(
            controllers::credit_card::upsert_credit_card(&card, &mut *tx),
            {
                not_created.push(card.id.clone());
            }
        );
    }

    macros::commit_transaction!(tx);

    let mut success = true;
    let mut message = String::from("ok");
    let status_code = build_status_code_for_multiple_input(
        body.len(),
        not_created.len(),
        &mut message,
        &mut success,
    );

    HttpResponse::build(status_code)
        .insert_header(ContentType::json())
        .body(
            json!({
                "success": success,
                "message": message,
                "errors": not_created
            })
            .to_string(),
        )
}

pub async fn delete_credit_card(
    req: HttpRequest,
    body: web::Json<Vec<DeleteCreditCardReq>>,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let ext = req.extensions();
    let session = macros::get_session!(ext);
    let mut con = macros::get_database_connection!(app_state);
    let user = macros::unwrap_opt_or_unauthorize!(macros::run_async_unwrap!(
        get_user(session.user_email.as_str(), &mut *con,),
        "an error ocurred when tried to get user from database"
    ));

    let mut tx = macros::begin_transaction!(con);
    let mut not_deleted: Vec<Uuid> = Vec::new();

    for req in body.iter() {
        macros::run_async_or!(
            controllers::credit_card::delete_credit_card(req.credit_card_id, user.id, &mut *tx),
            {
                not_deleted.push(req.credit_card_id);
            }
        )
    }

    macros::commit_transaction!(tx);
    let mut success = true;
    let mut message = String::from("ok");
    let status_code = build_status_code_for_multiple_input(
        body.len(),
        not_deleted.len(),
        &mut message,
        &mut success,
    );

    HttpResponse::build(status_code)
        .insert_header(ContentType::json())
        .body(
            json!({
                "success": success,
                "message": message,
                "errors": not_deleted
            })
            .to_string(),
        )
}

pub async fn list_credit_card(
    req: HttpRequest,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let ext = req.extensions();
    let session = macros::get_session!(ext);
    let mut con = macros::get_database_connection!(app_state);
    let user = macros::unwrap_opt_or_unauthorize!(macros::run_async_unwrap!(
        get_user(session.user_email.as_str(), &mut *con,),
        "an error occurred when tried to get user from database"
    ));

    let cards = macros::run_async_unwrap!(
        controllers::credit_card::get_credit_cards_by_user_id(user.id, &mut *con),
        "an error occurred when tried to get the credit cards from DB"
    );

    let mut res: Vec<ListCreditCardsRes> = Vec::new();
    for card in cards {
        res.push(ListCreditCardsRes {
            id: card.id.clone(),
            name: card.name.clone(),
            icon_name: card.icon_name.clone(),
            limit_value: card.limit_value,
            closing_day: card.closing_day,
        });
    }

    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(json!(res).to_string())
}

pub async fn create_bill_of_date(
    req: HttpRequest,
    body: web::Json<CreateBillAtDateReq>,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let ext = req.extensions();
    let session = macros::get_session!(ext);
    let mut con = macros::get_database_connection!(app_state);
    let user = macros::unwrap_opt_or_unauthorize!(macros::run_async_unwrap!(
        get_user(session.user_email.as_str(), &mut *con,),
        "an error occurred when tried to get user from database"
    ));

    let has_card = macros::run_async_unwrap!(
        controllers::credit_card::get_credit_card_by_id(&body.credit_card_id, user.id, &mut *con),
        "an error occurred when tried to check if credit card id exists for user"
    );

    let card = match has_card {
        Some(c) => c,
        None => {
            return HttpResponse::build(StatusCode::NOT_FOUND)
                .insert_header(ContentType::json())
                .body(json!({ "success": false, "message": "credit card not found"}).to_string());
        }
    };

    let date = match DateTime::parse_from_rfc3339(body.date.as_str()) {
        Ok(d) => d,
        Err(_) => {
            return HttpResponse::build(StatusCode::BAD_REQUEST)
                .insert_header(ContentType::json())
                .body(json!({ "success": false, "message": "invalid date"}).to_string());
        }
    };

    let bill = macros::run_async_unwrap!(
        controllers::credit_card::create_bill_of_date(
            &body.credit_card_id,
            card.closing_day,
            date.date_naive(),
            *date.offset(),
            &mut *con,
        ),
        "an error occurred when tried to create a bill for credit card"
    );

    let res = CreateBillRes {
        id: bill.id,
        credit_card_id: bill.credit_card_id,
        start_at: bill.start_at.to_rfc3339(),
        end_at: bill.end_at.to_rfc3339(),
    };

    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(json!(res).to_string())
}

pub async fn list_credit_card_bills(
    req: HttpRequest,
    body: web::Json<ListCreditCardBillsReq>,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let ext = req.extensions();
    let session = macros::get_session!(ext);
    let mut con = macros::get_database_connection!(app_state);
    let user = macros::unwrap_opt_or_unauthorize!(macros::run_async_unwrap!(
        get_user(session.user_email.as_str(), &mut *con,),
        "an error occurred when tried to get user from database"
    ));

    let bills = macros::run_async_unwrap!(
        controllers::credit_card::get_credit_card_bills(&body.credit_card_id, user.id, &mut *con),
        "an error occurred when tried to get bills from database"
    );
    let mut res: Vec<CreateBillRes> = Vec::new();

    for bill in bills {
        res.push(CreateBillRes {
            id: bill.id,
            credit_card_id: bill.credit_card_id,
            start_at: bill.start_at.to_rfc3339(),
            end_at: bill.end_at.to_rfc3339(),
        });
    }

    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(json!(res).to_string())
}
