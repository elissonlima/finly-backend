mod database;
mod model;
mod request_types;
mod routes;
mod state;
mod handlers;

use actix_web::{web, App, HttpServer};
use state::AppState;
use std::env;
use std::io;

#[actix_rt::main]
async fn main() -> io::Result<()> {
    dotenv::dotenv().ok();

    let database_path = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let ddl_script_path = env::var("DDL_PATH").expect("DDL_PATH is not set in .env file");

    // Setting Log configuration
    let env = env_logger::Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let db = database::DbConnection::build(database_path.as_str(), ddl_script_path.as_str()).await;
    log::info!("Started connection pool with database");

    let shared_data = web::Data::new(AppState { db: db.pool });

    let app = move || {
        App::new()
            .app_data(shared_data.clone())
            .configure(routes::auth_routes)
    };
    HttpServer::new(app).bind("127.0.0.1:3000")?.run().await
}
