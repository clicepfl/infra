use std::sync::Mutex;

use actix_web::{web, App, HttpServer};
use config::config;
use log::LogWriter;
use routes::{all, targeted};
use tracing::Level;

mod config;
mod error;
mod github;
mod log;
mod restart;
mod routes;
mod validation;

#[derive(Debug, Default)]
pub struct WebhookState {
    pub processed_deliveries: Vec<String>,
}
pub type State = Mutex<WebhookState>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(|| LogWriter {})
        .with_max_level(Level::DEBUG)
        .init();
    tracing::info!("Starting webhook on 127.0.0.1:4001");

    // Load the config
    config();
    let data = web::Data::<Mutex<WebhookState>>::default();

    HttpServer::new(move || {
        App::new()
            .service(all)
            .service(targeted)
            .app_data(data.clone())
    })
    .bind(("127.0.0.1", 4001))?
    .run()
    .await
}
