use actix_web::{middleware::from_fn, web};

use crate::{handler, middleware};

pub fn auth(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/gsignin", web::post().to(handler::google_signin))
            .route("/refresh_token", web::post().to(handler::refresh_token)),
    );
}

pub fn chat(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/chat")
            .wrap(from_fn(middleware::auth_middleware))
            .route("/send", web::post().to(handler::message_recv)),
    );
}

pub fn category(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/cat")
            .wrap(from_fn(middleware::auth_middleware))
            .route("", web::get().to(handler::list_categories))
            .route("/icons", web::get().to(handler::list_category_icons))
            .route("/colors", web::get().to(handler::list_category_colors))
            .route("", web::post().to(handler::create_category))
            .route("", web::patch().to(handler::update_category))
            .route("", web::delete().to(handler::delete_category))
    );
}



