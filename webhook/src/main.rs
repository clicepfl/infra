use actix_web::{App, HttpServer};
use config::config;
use routes::{generic, targeted};

mod config;
mod error;
mod models;
mod routes;
mod validation;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("Starting webhook on 127.0.0.1:8080");

    // Load the config
    config();

    HttpServer::new(|| App::new().service(generic).service(targeted))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
