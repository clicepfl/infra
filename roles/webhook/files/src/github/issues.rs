use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct PostIssueBody {
    pub title: String,
    pub body: String,
    pub assignees: Vec<String>,
}
#[derive(Serialize, Deserialize)]
pub struct EmptyBody {}

#[derive(Deserialize, Clone)]
pub struct OpenIssueBody {
    pub number: u32,
    pub title: String,
}

#[derive(Serialize)]
pub struct IssueCommentBody {
    pub body: String,
}

#[derive(Serialize)]
pub struct UpdateIssueBody {
    pub state: String,
}
