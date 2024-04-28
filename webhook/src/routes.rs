use std::process::Command;

use actix_web::{
    http::StatusCode,
    post,
    rt::spawn,
    web::{self, Payload},
    HttpRequest, HttpResponse,
};

use crate::{
    config::{config, Service},
    error::Error,
    validation::validate_call,
    State,
};

fn try_run(command: Option<&String>, service: &str) {
    let Some(command) = command else {
        return;
    };

    log::trace!("Running \"{}\"", command);

    match Command::new("sh")
        .args(["-c", &command])
        .env("SERVICE", service)
        .output()
    {
        Ok(r) if !r.status.success() => log::error!(
            "Command '{}' failed with {}\nSTDOUT:\n{}STDERR:\n{}",
            command,
            r.status,
            String::from_utf8(r.stdout).unwrap_or("<Unable to parse to utf-8 string>".to_owned()),
            String::from_utf8(r.stderr).unwrap_or("<Unable to parse to utf-8 string>".to_owned())
        ),
        Ok(r) if !r.status.success() => log::debug!(
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

fn restart(name: &str, service: &Service, default: &Service) {
    log::info!("Restarting service {name}");

    try_run(
        service
            .stop_command
            .as_ref()
            .or(default.stop_command.as_ref()),
        name,
    );
    try_run(
        service
            .pre_start_command
            .as_ref()
            .or(default.pre_start_command.as_ref()),
        name,
    );
    try_run(
        service
            .start_command
            .as_ref()
            .or(default.start_command.as_ref()),
        name,
    );
    try_run(
        service
            .post_start_command
            .as_ref()
            .or(default.post_start_command.as_ref()),
        name,
    );

    log::info!("Service {name} restarted");
}

#[post("/")]
pub async fn all(
    req: HttpRequest,
    payload: Payload,
    state: web::Data<State>,
) -> Result<HttpResponse<String>, Error> {
    let payload = payload.to_bytes().await?;
    if !validate_call(req.headers(), &payload, &mut state.lock().unwrap())? {
        return Ok(HttpResponse::with_body(StatusCode::OK, "OK".to_owned()));
    }

    spawn(async {
        log::info!("Triggering global restart");

        config()
            .services
            .iter()
            .for_each(|(n, s)| restart(n, s, &config().default));

        log::info!("Full restart complete");
    });

    Ok(HttpResponse::with_body(StatusCode::OK, "OK".to_owned()))
}

#[post("/{service}")]
pub async fn targeted(
    req: HttpRequest,
    payload: Payload,
    service: web::Path<String>,
    state: web::Data<State>,
) -> Result<HttpResponse<String>, Error> {
    let payload = payload.to_bytes().await?;
    if !validate_call(req.headers(), &payload, &mut state.lock().unwrap())? {
        return Ok(HttpResponse::with_body(StatusCode::OK, "OK".to_owned()));
    }

    log::info!("Triggering restart for service {}", service);

    if let Some(s) = config().services.get(service.as_str()) {
        restart(&service, s, &config().default);
        Ok(HttpResponse::with_body(StatusCode::OK, "OK".to_owned()))
    } else {
        log::warn!("Service {} not found", service);
        Err(Error::ServiceNotFound)
    }
}
