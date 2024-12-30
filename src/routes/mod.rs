use actix_web::web;

use crate::{
    handlers::{auth::*, misc::ping},
    middleware::Authentication,
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
            .wrap(Authentication)
            .route("/refresh", web::post().to(refresh_token)),
    );
}

pub fn misc_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/ping")
            .wrap(Authentication)
            .route("", web::get().to(ping)),
    );
}
