use actix_web::http::header::HeaderMap;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::{config::config, error::Error};

fn validate_signature(headers: &HeaderMap, payload: &[u8]) -> Result<(), Error> {
    let Some(Ok(Some(signature))) = headers
        .get("X-Hub-Signature-256")
        .map(|h| h.to_str().map(|s| s.strip_prefix("sha256=")))
    else {
        log::warn!(
            "Received request with badly formatted signature (origin: {:?})",
            headers.get("X-Forwarded-For")
        );
        return Err(Error::InvalidSignature);
    };

    let Ok(signature) = hex::decode(signature) else {
        log::warn!(
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
        log::warn!(
            "Received request with invalid signature (origin: {:?})",
            headers.get("X-Forwarded-For")
        );
        Err(Error::InvalidSignature)
    }
}

fn validate_event(headers: &HeaderMap) -> Result<bool, Error> {
    Ok(headers.get("X-GitHub-Event").is_some_and(|h| h != "ping"))
}

pub fn validate_call(headers: &HeaderMap, payload: &[u8]) -> Result<bool, Error> {
    validate_signature(headers, payload)?;
    validate_event(headers)
}
