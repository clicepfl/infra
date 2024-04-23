use std::process::Command;

use actix_web::{
    http::StatusCode,
    post,
    web::{self, Payload},
    HttpRequest, HttpResponse,
};

use crate::{config::config, error::Error, models::PushPayload, validation::validate_call};

fn try_run(command: &str) {
    match Command::new("sh").args(["-c", &command]).output() {
        Ok(r) if !r.status.success() => log::error!(
            "Command '{}' failed with {}\nSTDOUT:\n{}STDERR:\n{}",
            command,
            r.status,
            String::from_utf8(r.stdout).unwrap_or("<Unable to parse to utf-8 string>".to_owned()),
            String::from_utf8(r.stderr).unwrap_or("<Unable to parse to utf-8 string>".to_owned())
        ),
        Err(e) => log::error!("Unable to start command '{command}': {e:?}",),
        _ => {}
    }
}

#[post("/")]
pub async fn generic(req: HttpRequest, payload: Payload) -> Result<HttpResponse<String>, Error> {
    let payload = payload.to_bytes().await?;
    validate_call(req.headers(), &payload)?;
    let payload = serde_json::from_slice::<PushPayload>(&payload)?;

    log::info!(
        "Triggering global restart from commit [{}] (pushed by @{:?})",
        payload.after,
        payload.pusher.email
    );

    for service in config().services.iter() {
        log::info!("Restarting service {}", service.0);
        if let Some(cmd) = service.1.stop_command.as_ref() {
            try_run(cmd)
        }
        if let Some(cmd) = service.1.start_command.as_ref() {
            try_run(cmd)
        }
    }

    try_run(&config().generic_start_command);

    log::info!("Full restart complete");

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
    let payload = serde_json::from_slice::<PushPayload>(&payload)?;

    log::info!(
        "Triggering restart for service {} from commit [{}] (pushed by @{:?})",
        service,
        payload.after,
        payload.pusher.email
    );

    if let Some(service) = config().services.get(service.as_str()) {
        if let Some(cmd) = service.stop_command.as_ref() {
            try_run(cmd)
        }
        if let Some(cmd) = service.start_command.as_ref() {
            try_run(cmd)
        }

        log::info!("Partial restart complete");
        Ok(HttpResponse::with_body(StatusCode::OK, "OK".to_owned()))
    } else {
        log::warn!("Service {} not found", service);
        Err(Error::ServiceNotFound)
    }
}
