#![allow(unused)]
use serde::Deserialize;

#[derive(Deserialize)]
pub struct User {
    pub date: String,
    pub email: Option<String>,
    pub name: String,
    pub username: String,
}

#[derive(Deserialize)]
pub struct PushPayload {
    pub after: String,
    pub base_ref: Option<String>,
    pub before: String,
    pub compare: String,
    pub created: bool,
    pub deleted: bool,
    pub force: bool,
    pub pusher: User,
    #[serde(rename = "ref")]
    pub ref_: String,
}
