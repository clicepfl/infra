use std::time::SystemTime;

use actix_web::http::header::HeaderMap;
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;

use crate::{config::config, error::Error, WebhookState};

#[derive(Deserialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case")]
enum Action {
    Published { package: Package },
}
#[derive(Deserialize, Debug)]
struct Package {
    name: String,
}

fn validate_delivery(payload: &[u8], state: &mut WebhookState) -> Result<bool, Error> {
    if let Ok(Action::Published { package }) = serde_json::from_slice::<Action>(payload) {
        if let Some(date) = state.processed_packages.get(&package.name) {
            if date.elapsed().is_ok_and(|d| d.as_secs() < 120) {
                tracing::info!(
                    "Already triggered recently ({}), skipping",
                    date.duration_since(SystemTime::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0)
                );
                return Ok(false);
            }
        }

        state
            .processed_packages
            .insert(package.name, SystemTime::now());
    }

    Ok(true)
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
    Ok(validate_event(headers)? && validate_delivery(payload, state)?)
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
