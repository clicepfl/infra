#![allow(unused)]
use serde::Deserialize;

#[derive(Deserialize)]
pub struct User {
    pub email: Option<String>,
    pub name: String,
}

#[derive(Deserialize)]
pub struct PushPayload {
    pub after: String,
    pub pusher: User,
    #[serde(rename = "ref")]
    pub ref_: String,
}
