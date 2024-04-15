use actix_web::{
    http::StatusCode,
    post,
    web::{self, Payload},
    App, HttpRequest, HttpResponse, HttpServer,
};

use crate::{error::Error, validation::validate_call};

mod error;
mod models;
mod validation;

#[post("/hello/{name}")]
async fn greet(
    name: web::Path<String>,
    req: HttpRequest,
    payload: Payload,
) -> Result<HttpResponse<String>, Error> {
    let payload = payload.to_bytes().await?;

    validate_call(req.headers(), &payload)?;

    Ok(HttpResponse::with_body(
        StatusCode::OK,
        format!("Hello {name}!"),
    ))
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(greet))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
