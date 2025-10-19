use actix_web::http::header::HeaderMap;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::{
    config::config,
    error::Error,
    github::event::{parse_payload, Payload, HEADER_DELIVERY_ID},
    WebhookState,
};

fn validate_event(headers: &HeaderMap, payload: &[u8]) -> Result<bool, Error> {
    match parse_payload(headers, payload)? {
        Payload::Action(_) => {
            // All publicated packages trigger a redeploy.
            Ok(true)
        }
        Payload::Push(push) => {
            // Only pushes to the repo main branch should trigger a redeploy.
            Ok(push.r#ref == format!("refs/head/{}", push.repository.default_branch))
        }
    }
}

fn validate_signature(headers: &HeaderMap, payload: &[u8]) -> Result<bool, Error> {
    let Some(Ok(Some(signature))) = headers
        .get("X-Hub-Signature-256")
        .map(|h| h.to_str().map(|s| s.strip_prefix("sha256=")))
    else {
        tracing::warn!(
            "Received request with badly formatted signature (origin: {:?})",
            headers.get("X-Forwarded-For")
        );
        return Err(Error::InvalidSignature);
    };

    let Ok(signature) = hex::decode(signature) else {
        tracing::warn!(
            "Received request with badly formatted signature (origin: {:?})",
            headers.get("X-Forwarded-For")
        );
        return Err(Error::InvalidSignature);
    };

    let mut hmac = Hmac::<Sha256>::new_from_slice(config().secret.as_bytes()).unwrap();
    hmac.update(payload);

    if hmac.verify_slice(&signature).is_ok() {
        Ok(true)
    } else {
        tracing::warn!(
            "Received request with invalid signature (origin: {:?})",
            headers.get("X-Forwarded-For")
        );
        Err(Error::InvalidSignature)
    }
}

fn check_redelivery(headers: &HeaderMap, state: &mut WebhookState) -> Result<bool, Error> {
    let Some(Ok(delivery_id)) = headers.get(HEADER_DELIVERY_ID).map(|h| h.to_str()) else {
        return Err(Error::BadRequest);
    };
    let delivery_id = delivery_id.to_owned();

    // Ignore deliveries that were already/are being processed.
    if state.processed_deliveries.contains(&delivery_id) {
        return Ok(false);
    }
    state.processed_deliveries.push(delivery_id);

    Ok(true)
}

pub fn validate_call(
    headers: &HeaderMap,
    payload: &[u8],
    state: &mut WebhookState,
) -> Result<bool, Error> {
    Ok(check_redelivery(headers, state)?
        && validate_signature(headers, payload)?
        && validate_event(headers, payload)?)
}

pub fn validate_service_list(str: &str) -> Result<(), Error> {
    let mut services = str.split(",");
    let allowed: Vec<&str> = config().services.keys().map(|s| s.as_ref()).collect();

    if services.all(|s| allowed.contains(&s)) {
        Ok(())
    } else {
        Err(Error::InvalidServiceList)
    }
}
