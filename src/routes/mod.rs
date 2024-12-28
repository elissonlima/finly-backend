use actix_web::web;

use crate::handlers::auth::post_new_user;

pub fn auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").route(
        "/create_user", 
        web::post().to(post_new_user)));
}
