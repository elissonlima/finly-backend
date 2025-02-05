use actix_web::{
    http::{header::ContentType, StatusCode},
    web, HttpMessage, HttpRequest, HttpResponse,
};
use serde_json::json;

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
    body: web::Json<UpsertCategoryReq>,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let ext = req.extensions();
    let session = macros::get_session!(ext);
    let mut con = macros::get_database_connection!(app_state);
    let user = macros::unwrap_opt_or_unauthorize!(macros::run_async_unwrap!(
        get_user(session.user_email.as_str(), &mut *con,),
        "An error ocurred when tried to get user from database"
    ));

    let cat = Category {
        id: body.id.clone(),
        user_id: user.id,
        name: body.name.clone(),
        color: body.color.clone(),
        icon_name: body.icon_name.clone(),
        subcategories: Vec::new(),
    };

    macros::run_async_unwrap!(
        controllers::category::upsert_category(cat, &mut *con),
        "an error occurred when tried to create category"
    );

    HttpResponse::build(StatusCode::CREATED)
        .insert_header(ContentType::json())
        .body(json!({"success": true}).to_string())
}

pub async fn delete_category(
    req: HttpRequest,
    body: web::Json<DeleteCategoryReq>,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let ext = req.extensions();
    let session = macros::get_session!(ext);
    let mut con = macros::get_database_connection!(app_state);
    let user = macros::unwrap_opt_or_unauthorize!(macros::run_async_unwrap!(
        get_user(session.user_email.as_str(), &mut *con,),
        "an error ocurred when tried to get user from database"
    ));

    macros::run_async_unwrap!(
        controllers::category::delete_category(body.category_id.as_str(), user.id, &mut *con),
        "an error occurred when tried to update the category"
    );

    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(json!({"success": true}).to_string())
}

pub async fn upsert_subcategory(
    body: web::Json<UpsertSubcategoryReq>,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let mut con = macros::get_database_connection!(app_state);

    let sub = Subcategory {
        id: body.id.clone(),
        category_id: body.category_id.clone(),
        name: body.name.clone(),
        icon_name: body.icon_name.clone(),
        color: body.color.clone(),
    };

    macros::run_async_unwrap!(
        controllers::category::upsert_subcategory(sub, &mut *con),
        "an error occurred when tried to upsert subcategory"
    );

    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(json!({"success": true}).to_string())
}

pub async fn delete_subcategory(
    body: web::Json<DeleteSubcategoryReq>,
    app_state: web::Data<state::AppState>,
) -> HttpResponse {
    let mut con = macros::get_database_connection!(app_state);

    macros::run_async_unwrap!(
        controllers::category::delete_subcategory(
            body.subcategory_id.as_str(),
            body.category_id.as_str(),
            &mut *con
        ),
        "an error occurred when tried to update subcategory record"
    );

    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(json!({"success": true}).to_string())
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
        let sub = subcategories.entry(cat.id.clone()).or_default();
        cat.subcategories.extend(sub.to_vec());
        res.push(cat);
    }

    HttpResponse::build(StatusCode::OK)
        .insert_header(ContentType::json())
        .body(json!(res).to_string())
}
