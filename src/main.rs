mod app_state;
mod args;
mod controller;
mod handler;
mod jwt;
mod middleware;
mod model;
mod route;

use std::{fs, sync::Mutex};

use actix_web::{
    App, HttpServer,
    middleware::{Compress, Logger},
    web,
};
use args::Args;
use clap::Parser;
use jsonwebtoken::{DecodingKey, EncodingKey};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use sqlx::PgPool;

use crate::{app_state::AppState, controller::get_access_token, model::ServiceAccountKey};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Handle Command Line Arguments
    let args = Args::parse();

    // Log Configuration
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    log::info!("Starting finly-backend");

    // Opening Database connection
    let pool = PgPool::connect(&args.dburl)
        .await
        .expect("It wasn't possible to open connection pool with database");

    // JWT Keys
    let jwt_enc_key = EncodingKey::from_rsa_pem(
        &std::fs::read(args.jwt_en_key).expect("Could not load JWT Encoding Key from File"),
    )
    .expect("It wasn't possible to create the JWT encoding key");
    let jwt_dec_key = DecodingKey::from_rsa_pem(
        &std::fs::read(args.jwt_de_key).expect("Could not load JWT Decoding Key from File"),
    )
    .expect("It wasn't possible to create the JWT decoding key");

    //Load Google Service Account Key
    let key_json = fs::read_to_string(args.google_key)
        .expect("It wasn't possible to load Google Service Key File");
    let google_key: ServiceAccountKey = serde_json::from_str(&key_json)
        .expect("Error while trying convert Google Service Key from string");
    let google_service_token = get_access_token(&google_key)
        .await
        .expect("Could not retrive the google service access token");

    // App State
    let app_state = web::Data::new(AppState {
        pool,
        jwt_encoding_key: jwt_enc_key,
        jwt_decoding_key: jwt_dec_key,
        google_service_token: Mutex::new(google_service_token),
        google_service_key: google_key
    });

    // load TLS keys
    let mut ssl_builder =
        SslAcceptor::mozilla_intermediate(SslMethod::tls()).expect("Couldn't created SSL Builder");

    ssl_builder
        .set_private_key_file(args.tls_key, SslFiletype::PEM)
        .expect("It wasn't possible to open the TLS private key");

    ssl_builder
        .set_certificate_chain_file(args.tls_cert)
        .expect("It wasn't possible to open the TLS Certificate");

    let app = move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(Logger::default())
            .wrap(Compress::default())
            .configure(route::auth)
            .configure(route::chat)
    };

    HttpServer::new(app)
        .bind_openssl("0.0.0.0:8080", ssl_builder)?
        //.bind("0.0.0.0:8080")?
        .workers(1)
        .run()
        .await
}
