#![allow(unused)]
use serde::Deserialize;

#[derive(Deserialize)]
pub struct User {
    date: String,
    email: Option<String>,
    name: String,
    username: String,
}

#[derive(Deserialize)]
pub struct PushPayload {
    after: String,
    base_ref: Option<String>,
    before: String,
    compare: String,
    created: bool,
    deleted: bool,
    force: bool,
    pusher: User,
    #[serde(rename = "ref")]
    ref_: String,
}
