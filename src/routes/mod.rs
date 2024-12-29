use actix_web::web;

use crate::handlers::auth::*;

pub fn auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/create_user", web::post().to(post_new_user))
            .route("/login", web::post().to(login_user)),
    );
}
