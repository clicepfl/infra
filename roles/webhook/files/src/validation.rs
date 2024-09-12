use actix_web::http::header::HeaderMap;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::{config::config, error::Error, WebhookState};

fn validate_delivery(headers: &HeaderMap, state: &mut WebhookState) -> Result<bool, Error> {
    let Some(delivery_id) = headers
        .get("X-GitHub-Delivery")
        .and_then(|h| h.to_str().ok())
    else {
        return Err(Error::InvalidDelivery);
    };

    if state.processed_deliveries.contains(&delivery_id.to_owned()) {
        tracing::info!("Ignoring re-delivery for {delivery_id}");
        Ok(false)
    } else {
        state.processed_deliveries.insert(0, delivery_id.to_owned());
        state.processed_deliveries.truncate(10);
        Ok(true)
    }
}

fn validate_signature(headers: &HeaderMap, payload: &[u8]) -> Result<(), Error> {
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
        Ok(())
    } else {
        tracing::warn!(
            "Received request with invalid signature (origin: {:?})",
            headers.get("X-Forwarded-For")
        );
        Err(Error::InvalidSignature)
    }
}

fn validate_event(headers: &HeaderMap) -> Result<bool, Error> {
    Ok(headers.get("X-GitHub-Event").is_some_and(|h| h != "ping"))
}

pub fn validate_call(
    headers: &HeaderMap,
    payload: &[u8],
    state: &mut WebhookState,
) -> Result<bool, Error> {
    validate_signature(headers, payload)?;
    Ok(validate_event(headers)? && validate_delivery(headers, state)?)
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
