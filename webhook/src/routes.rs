use std::process::Command;

use actix_web::{
    http::StatusCode,
    post,
    rt::spawn,
    web::{self, Payload},
    HttpRequest, HttpResponse,
};

use crate::{config::config, error::Error, validation::validate_call, State};

fn try_run(command: &str) {
    log::trace!("Running \"{}\"", command);

    match Command::new("sh").args(["-c", &command]).output() {
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

fn try_run_opt(command: &Option<String>) {
    if let Some(cmd) = command {
        try_run(cmd);
    }
}

#[post("/")]
pub async fn generic(
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

        for (name, service) in config().services.iter() {
            log::info!("Restarting service {}", name);
            try_run_opt(&service.stop_command);
            try_run_opt(&service.pre_start_command);
            try_run_opt(&service.start_command);
        }

        try_run(&config().generic_start_command);

        for (_, service) in config().services.iter() {
            try_run_opt(&service.post_start_command);
        }

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

    if let Some(service) = config().services.get(service.as_str()) {
        spawn(async {
            try_run_opt(&service.stop_command);
            try_run_opt(&service.start_command);

            if service.start_command.is_none() {
                try_run(&config().generic_start_command);
            }

            log::info!("Partial restart complete");
        });

        Ok(HttpResponse::with_body(StatusCode::OK, "OK".to_owned()))
    } else {
        log::warn!("Service {} not found", service);
        Err(Error::ServiceNotFound)
    }
}
