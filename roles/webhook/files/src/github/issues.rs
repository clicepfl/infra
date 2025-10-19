use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct PostIssueBody {
    pub title: String,
    pub body: String,
    pub assignees: Vec<String>,
}
#[derive(Serialize, Deserialize)]
pub struct EmptyBody {}
