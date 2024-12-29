mod controllers;
mod database;
mod handlers;
mod jwt;
mod model;
mod request_types;
mod routes;
mod state;

use actix_web::{web, App, HttpServer};
use jsonwebtoken::DecodingKey;
use jsonwebtoken::EncodingKey;
use state::AppState;
use std::env;
use std::fs;
use std::io;

#[actix_rt::main]
async fn main() -> io::Result<()> {
    dotenv::dotenv().ok();

    let database_path = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let ddl_script_path = env::var("DDL_PATH").expect("DDL_PATH is not set in .env file");
    let jwt_encoding_key_path =
        env::var("JWT_ENC_PATH").expect("JWT_ENC_PATH is not set in .env file");
    let jwt_decoding_key_path =
        env::var("JWT_DEC_PATH").expect("JWT_DEC_PATH is not set in .env file");

    // Setting Log configuration
    let env = env_logger::Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let db = database::DbConnection::build(database_path.as_str(), ddl_script_path.as_str()).await;
    log::info!("Started connection pool with database");

    let jwt_enc_key_fs = fs::read(jwt_encoding_key_path).unwrap();
    let jwt_dec_key_fs = fs::read(jwt_decoding_key_path).unwrap();
    let jwt_enc_key = EncodingKey::from_rsa_pem(&jwt_enc_key_fs).unwrap();
    let jwt_dec_key = DecodingKey::from_rsa_pem(&jwt_dec_key_fs).unwrap();

    let shared_data = web::Data::new(AppState {
        db: db.pool,
        jwt_encoding_key: jwt_enc_key,
        jwt_decoding_key: jwt_dec_key,
    });

    let app = move || {
        App::new()
            .app_data(shared_data.clone())
            .configure(routes::auth_routes)
    };
    HttpServer::new(app).bind("127.0.0.1:3000")?.run().await
}
