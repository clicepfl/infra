use std::time::SystemTime;

use actix_web::http::header::HeaderMap;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::{
    config::config,
    error::Error,
    github::event::{Action, Push},
    WebhookState,
};

fn validate_event(
    headers: &HeaderMap,
    payload: &[u8],
    state: &mut WebhookState,
) -> Result<bool, Error> {
    match headers
        .get("X-GitHub-Event")
        .map(|h| h.to_str().ok())
        .flatten()
    {
        Some("action") => {
            // All publicated packages trigger a redeploy unless the package was redeployed recently
            // enough (ergo this call is considered retry and should be ignored).
            let Action::Published { package } = serde_json::from_slice(payload)?;
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

            Ok(true)
        }
        Some("push") => {
            // Only pushes to the repo main branch should trigger a redeploy.
            let Push { r#ref, repository } = serde_json::from_slice(payload)?;
            Ok(r#ref == format!("refs/head/{}", repository.default_branch))
        }
        _ => Err(Error::ForbiddenEvent),
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

pub fn validate_call(
    headers: &HeaderMap,
    payload: &[u8],
    state: &mut WebhookState,
) -> Result<bool, Error> {
    validate_signature(headers, payload)?;
    validate_event(headers, payload, state)
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
