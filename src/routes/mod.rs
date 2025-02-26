use actix_files::{self as fs};
use actix_web::{middleware::from_fn, web};

use crate::{
    handlers::{
        auth::*,
        category::{
            delete_category, delete_subcategory, list_category, upsert_category, upsert_subcategory,
        },
        credit_card::{
            create_bill_of_date, delete_credit_card, list_credit_card, list_credit_card_bills,
            upsert_credit_card,
        },
        html::terms_of_use,
        reset_password::{create_reset_password_request, do_reset_password, reset_password_form},
        session_mgm::{logout_user, ping},
        static_content::file_list_handler,
    },
    middleware::{auth_middleware, refresh_token_middleware},
};

pub fn auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/create_user", web::post().to(post_new_user))
            .route("/login", web::post().to(login_user))
            .route("/google_signin", web::post().to(google_signin)),
    );
}

pub fn token_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/token")
            .wrap(from_fn(refresh_token_middleware))
            .route("/refresh", web::post().to(refresh_token)),
    );
}

pub fn session_mgm_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/session")
            .wrap(from_fn(auth_middleware))
            .route("/ping", web::get().to(ping))
            .route("/logout", web::post().to(logout_user)),
    );
}

pub fn html_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/html").route("/terms", web::get().to(terms_of_use)));
}

pub fn reset_password_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/password")
            .route(
                "/request_reset",
                web::post().to(create_reset_password_request),
            )
            .route("/reset", web::get().to(reset_password_form))
            .route("/reset", web::post().to(do_reset_password)),
    );
}

pub fn category_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/category")
            .wrap(from_fn(auth_middleware))
            .route("", web::post().to(upsert_category))
            .route("", web::delete().to(delete_category))
            .route("", web::get().to(list_category))
            .route("/sub", web::post().to(upsert_subcategory))
            .route("/sub", web::delete().to(delete_subcategory))
            .service(
                fs::Files::new("/icons", "./category_icons")
                    .files_listing_renderer(file_list_handler)
                    .show_files_listing(),
            ),
    );
}

pub fn credit_card_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/ccard")
            .wrap(from_fn(auth_middleware))
            .route("", web::post().to(upsert_credit_card))
            .route("", web::delete().to(delete_credit_card))
            .route("", web::get().to(list_credit_card))
            .route("/bill", web::get().to(list_credit_card_bills))
            .route("bill_of_date", web::post().to(create_bill_of_date))
            .service(
                fs::Files::new("/icons", "./card_icons")
                    .files_listing_renderer(file_list_handler)
                    .show_files_listing(),
            ),
    );
}
