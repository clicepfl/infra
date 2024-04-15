use actix_web::{get, http::StatusCode, web, App, Error, HttpMessage, HttpRequest, HttpResponse, HttpServer};

mod models;

fn validate_signature(req: &HttpRequest) -> bool {
    req.headers()
        .get("X-Hub-Signature")
        .is_some_and(|v| v == "1234")
}

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>, req: HttpRequest) -> Result<HttpResponse<String>, Error> {
    if !validate_signature(&req) {
        return HttpResponse::Forbidden().message_body("Forbidden".to_owned());
    }

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
