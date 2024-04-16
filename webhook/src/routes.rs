use std::process::Command;

use actix_web::{
    http::StatusCode,
    post,
    web::{self, Payload},
    HttpRequest, HttpResponse,
};

use crate::{config::config, error::Error, validation::validate_call};

fn try_run(command: &str) {
    if let Err(e) = Command::new("sh").args(["-c", &command]).output() {
        println!("Error while running '{command}':\n{e:?}",)
    }
}

#[post("/")]
pub async fn generic(req: HttpRequest, payload: Payload) -> Result<HttpResponse<String>, Error> {
    let payload = payload.to_bytes().await?;
    validate_call(req.headers(), &payload)?;

    for service in config().services.iter() {
        if let Some(cmd) = service.1.stop_command.as_ref() {
            try_run(cmd)
        }
        if let Some(cmd) = service.1.start_command.as_ref() {
            try_run(cmd)
        }
    }

    try_run(&config().generic_start_command);

    Ok(HttpResponse::with_body(StatusCode::OK, "OK".to_owned()))
}

#[post("/{service}")]
pub async fn targeted(
    req: HttpRequest,
    payload: Payload,
    service: web::Path<String>,
) -> Result<HttpResponse<String>, Error> {
    let payload = payload.to_bytes().await?;
    validate_call(req.headers(), &payload)?;

    if let Some(service) = config().services.get(service.as_str()) {
        if let Some(cmd) = service.stop_command.as_ref() {
            try_run(cmd)
        }
        if let Some(cmd) = service.start_command.as_ref() {
            try_run(cmd)
        }

        Ok(HttpResponse::with_body(StatusCode::OK, "OK".to_owned()))
    } else {
        Err(Error::ServiceNotFound)
    }

}
