use actix_web::{
    http::{header::ContentType, StatusCode},
    web, HttpMessage, HttpRequest, HttpResponse,
};
use serde_json::json;
use sqlx::Acquire;

use crate::{
    controllers::{self, auth::get_user},
    handlers::macros,
    model::{category::Category, subcategory::Subcategory},
    request_types::category::{
        DeleteCategoryReq, DeleteSubcategoryReq, UpsertCategoryReq, UpsertSubcategoryReq,
    },
    state,
};

pub async fn upsert_category(
    req: HttpRequest,
    body: web::Json<Vec<UpsertCategoryReq>>,
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

    for rec in body.iter() {
        let cat = Category {
            id: rec.id,
            user_id: user.id,
            name: rec.name.clone(),
            color: rec.color.clone(),
            icon_name: rec.icon_name.clone(),
            subcategories: Vec::new(),
        };

        macros::run_async_unwrap!(
            controllers::category::upsert_category(&cat, &mut *tx),
            "an error occurred when tried to create category"
        );
    }

    macros::commit_transaction!(tx);

    HttpResponse::build(StatusCode::CREATED)
        .insert_header(ContentType::json())
        .body(json!({"success": true, "message":"ok"}).to_string())
}

pub async fn delete_category(
    req: HttpRequest,
    body: web::Json<Vec<DeleteCategoryReq>>,
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

    for res in body.iter() {
        macros::run_async_unwrap!(
            controllers::category::delete_category(&res.category_id, user.id, &mut *tx),
            "an error occurred when tried to update the category"
        );
    }

    macros::commit_transaction!(tx);

    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(json!({"success": true, "message":"ok"}).to_string())
}

pub async fn upsert_subcategory(
    body: web::Json<Vec<UpsertSubcategoryReq>>,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let mut con = macros::get_database_connection!(app_state);
    let mut tx = macros::begin_transaction!(con);

    for rec in body.iter() {
        let sub = Subcategory {
            id: rec.id,
            category_id: rec.category_id,
            name: rec.name.clone(),
            icon_name: rec.icon_name.clone(),
            color: rec.color.clone(),
        };

        macros::run_async_unwrap!(
            controllers::category::upsert_subcategory(&sub, &mut *tx),
            "an error occurred when tried to upsert subcategory"
        );
    }

    macros::commit_transaction!(tx);

    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(json!({"success": true, "message":"ok"}).to_string())
}

pub async fn delete_subcategory(
    body: web::Json<Vec<DeleteSubcategoryReq>>,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let mut con = macros::get_database_connection!(app_state);
    let mut tx = macros::begin_transaction!(con);

    for rec in body.iter() {
        macros::run_async_unwrap!(
            controllers::category::delete_subcategory(
                &rec.subcategory_id,
                &rec.category_id,
                &mut *tx
            ),
            "an error occurred when tried to update subcategory record"
        );
    }

    macros::commit_transaction!(tx);

    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(json!({"success": true, "message":"ok"}).to_string())
}

pub async fn list_category(
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

    let categories = macros::run_async_unwrap!(
        controllers::category::get_categories_by_user_id(user.id, &mut *con),
        "an error occurred when tried to get the categories from DB"
    );
    let mut subcategories = macros::run_async_unwrap!(
        controllers::category::get_subcategories_by_user_id(user.id, &mut *con),
        "an error occurred when tried to get the subcategories from DB"
    );

    let mut res: Vec<Category> = Vec::new();

    for mut cat in categories {
        let sub = subcategories.entry(cat.id.to_string().clone()).or_default();
        cat.subcategories.extend(sub.to_vec());
        res.push(cat);
    }

    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(json!(res).to_string())
}
