use actix_web::{middleware::from_fn, web};

use crate::{
    handlers::{auth::*, html::terms_of_use, misc::ping},
    middleware::{auth_middleware, refresh_token_middleware},
};

pub fn auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/create_user", web::post().to(post_new_user))
            .route("/login", web::post().to(login_user)),
    );
}

pub fn token_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/token")
            .wrap(from_fn(refresh_token_middleware))
            .route("/refresh", web::post().to(refresh_token)),
    );
}

pub fn misc_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/ping")
            .wrap(from_fn(auth_middleware))
            .route("", web::get().to(ping)),
    );
}

pub fn html_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/html").route("/terms", web::get().to(terms_of_use)));
}
