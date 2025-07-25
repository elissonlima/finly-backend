use actix_web::web;

use crate::handler;

pub fn auth(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").route("/login", web::post().to(handler::auth_login)));
}

