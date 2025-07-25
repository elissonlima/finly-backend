mod app_state;
mod args;
mod controller;
mod handler;
mod model;
mod route;

use actix_web::{
    App, HttpServer,
    middleware::{Compress, Logger},
    web,
};
use args::Args;
use clap::Parser;
use sqlx::PgPool;

use crate::app_state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Handle Command Line Arguments
    let args = Args::parse();

    // Log Configuration
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    log::info!("Starting finly-backend");

    let pool = PgPool::connect(&args.dburl)
        .await
        .expect("It wasn't possible to open connection pool with database");
    let app_state = web::Data::new(AppState { pool });

    let app = move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(Logger::default())
            .wrap(Compress::default())
            .configure(route::auth)
    };

    HttpServer::new(app)
        .bind("0.0.0.0:8080")?
        .workers(1)
        .run()
        .await
}
