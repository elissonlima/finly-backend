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
