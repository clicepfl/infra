use crate::{
    config::config,
    error::Error,
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

    spawn(async {
        tracing::info!("Triggered global restart");
        start_capture();
        tracing::info!("Triggered global restart");

        config()
            .services
            .iter()
            .for_each(|(n, s)| restart(n, s, &config().default));

        tracing::info!("Full restart complete");

        let log = stop_capture();
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

        let res = if let Some(s) = config().services.get(service.as_str()) {
            restart(&service, s, &config().default);
            Ok(HttpResponse::with_body(StatusCode::OK, "OK".to_owned()))
        } else {
            tracing::warn!("Service {} not found", service);
            Err(Error::ServiceNotFound)
        };

        let log = stop_capture();
    });

    Ok(HttpResponse::with_body(StatusCode::OK, "OK".to_owned()))
}
