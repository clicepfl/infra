use actix_web::http::header::HeaderMap;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::{config::config, error::Error, models::PushPayload};

fn validate_signature(headers: &HeaderMap, payload: &[u8]) -> Result<(), Error> {
    let Some(Ok(Some(signature))) = headers
        .get("X-Hub-Signature-256")
        .map(|h| h.to_str().map(|s| s.strip_prefix("sha256=")))
    else {
        println!("Unable to get the signature");
        return Err(Error::InvalidSignature);
    };

    let Ok(signature) = hex::decode(signature) else {
        println!("Unable to decode the signature");
        return Err(Error::InvalidSignature);
    };

    let mut hmac = Hmac::<Sha256>::new_from_slice(config().secret.as_bytes()).unwrap();
    hmac.update(payload);

    if hmac.verify_slice(&signature).is_ok() {
        Ok(())
    } else {
        Err(Error::InvalidSignature)
    }
}

fn validate_event(headers: &HeaderMap, payload: &[u8]) -> Result<(), Error> {
    if !headers.get("X-GitHub-Event").is_some_and(|h| h == "push") {
        return Err(Error::InvalidEvent);
    }

    let payload: PushPayload = serde_json::from_slice(payload)?;

    if payload.ref_ == config().ref_ {
        Ok(())
    } else {
        Err(Error::InvalidRef)
    }
}

pub fn validate_call(headers: &HeaderMap, payload: &[u8]) -> Result<(), Error> {
    validate_signature(headers, payload)?;
    validate_event(headers, payload)?;
    Ok(())
}
