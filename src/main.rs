mod controllers;
mod database;
mod handlers;
mod jwt;
mod middleware;
mod model;
mod request_types;
mod routes;
mod state;

use actix_web::middleware::{Compress, Logger};
use actix_web::{web, App, HttpServer};
use jsonwebtoken::DecodingKey;
use jsonwebtoken::EncodingKey;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use state::AppState;
use std::env;
use std::fs;
use std::io;

#[actix_rt::main]
async fn main() -> io::Result<()> {
    dotenv::dotenv().ok();

    let database_path = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let jwt_encoding_key_path =
        env::var("JWT_ENC_PATH").expect("JWT_ENC_PATH is not set in .env file");
    let jwt_decoding_key_path =
        env::var("JWT_DEC_PATH").expect("JWT_DEC_PATH is not set in .env file");
    let tls_key_path = env::var("TLS_KEY_PATH").expect("TLS_KEY_PATH is not set");
    let tls_cert_path = env::var("TLS_CERT_PATH").expect("TLS_CERT_PATH is not set");
    let session_db_ddl_script_path =
        env::var("SESSION_DB_DDL_SCRIPT_PATH").expect("SESSION_DB_DDL_SCRIPT_PATH is not set");

    // Setting Log configuration
    let env = env_logger::Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    log::info!("Starting server");

    let db = database::DbConnection::build(database_path.as_str()).await;
    let session_db =
        database::DbConnection::build_in_memory(session_db_ddl_script_path.as_str()).await;
    log::info!("Started connection pool with database");

    let jwt_enc_key_fs = fs::read(jwt_encoding_key_path).unwrap();
    let jwt_dec_key_fs = fs::read(jwt_decoding_key_path).unwrap();
    let jwt_enc_key = EncodingKey::from_rsa_pem(&jwt_enc_key_fs).unwrap();
    let jwt_dec_key = DecodingKey::from_rsa_pem(&jwt_dec_key_fs).unwrap();

    let shared_data = web::Data::new(AppState {
        db: db.pool,
        session_db: session_db.pool,
        jwt_encoding_key: jwt_enc_key,
        jwt_decoding_key: jwt_dec_key,
    });

    // load TLS keys
    let mut ssl_builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    ssl_builder
        .set_private_key_file(tls_key_path, SslFiletype::PEM)
        .unwrap();
    ssl_builder
        .set_certificate_chain_file(tls_cert_path)
        .unwrap();

    let app = move || {
        App::new()
            .app_data(shared_data.clone())
            .wrap(Logger::default())
            .wrap(Compress::default())
            .configure(routes::auth_routes)
            .configure(routes::token_routes)
            .configure(routes::misc_routes)
            .configure(routes::html_routes)
    };

    HttpServer::new(app)
        .bind_openssl("0.0.0.0:3000", ssl_builder)?
        .workers(4)
        .run()
        .await
}
