use crate::{
    config::config,
    error::Error,
    github::{close_issues, open_issue},
    log::{start_capture, stop_capture},
    restart::restart,
    validation::{validate_call, validate_service_list},
    State,
};
use actix_web::{
    http::StatusCode,
    post,
    rt::spawn,
    web::{self, Payload},
    HttpRequest, HttpResponse,
};

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

    spawn(async move {
        tracing::info!("Triggered global restart");
        start_capture();
        tracing::info!("Triggered global restart");

        let mut failed = false;

        for (n, s) in config().services.iter() {
            if !restart(n, s, &config().default) {
                failed = true;
            }
        }

        tracing::info!("Full restart complete");

        let log = stop_capture();
        let payload = String::from_utf8_lossy(&payload).to_string();

        if failed {
            open_issue(log, vec![], payload).await;
        } else {
            close_issues(config().services.keys().cloned().collect(), payload).await;
        }
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
    validate_service_list(&service)?;

    spawn(async move {
        tracing::info!("Triggered restart for service {}", service);
        start_capture();
        tracing::info!("Triggered restart for service {}", service);

        let mut failed = false;

        if let Some(s) = config().services.get(service.as_str()) {
            if !restart(&service, s, &config().default) {
                failed = true
            }
        } else {
            tracing::warn!("Service {} not found", service);
        };

        let log = stop_capture();
        let payload = String::from_utf8_lossy(&payload).to_string();

        if failed {
            open_issue(log, vec![service.to_string()], payload).await;
        } else {
            close_issues(vec![service.to_string()], payload).await;
        }
    });

    Ok(HttpResponse::with_body(StatusCode::OK, "OK".to_owned()))
}
