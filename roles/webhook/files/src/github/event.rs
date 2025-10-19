use actix_web::http::header::HeaderMap;
use serde::Deserialize;

use crate::error::Error;

#[derive(Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub updated_at: Option<String>,
    pub html_url: String,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
    Published { package: Package },
}

#[derive(Deserialize, Debug)]
pub struct Repository {
    pub html_url: String,
    pub default_branch: String,
}

#[derive(Deserialize, Debug)]
pub struct Commit {}

#[derive(Deserialize, Debug)]
pub struct Push {
    pub after: String,
    pub repository: Repository,
    pub commits: Vec<Commit>,
    #[serde(rename = "ref")]
    pub r#ref: String,
}

pub const HEADER_EVENT: &str = "X-GitHub-Event";
pub const HEADER_DELIVERY_ID: &str = "X-GitHub-Delivery";

pub enum Payload {
    Action(Action),
    Push(Push),
}

pub fn parse_payload(headers: &HeaderMap, payload: &[u8]) -> Result<Payload, Error> {
    match headers.get(HEADER_EVENT).and_then(|h| h.to_str().ok()) {
        Some("action") => Ok(Payload::Action(serde_json::from_slice(payload)?)),
        Some("push") => Ok(Payload::Push(serde_json::from_slice(payload)?)),
        _ => Err(Error::ForbiddenEvent),
    }
}
