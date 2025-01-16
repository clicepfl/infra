use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::config;

#[derive(Serialize)]
struct PostIssueBody {
    title: String,
    body: String,
    assignees: Vec<String>,
}

#[derive(Deserialize)]
struct Repository {
    html_url: String,
}

#[derive(Deserialize)]
struct Package {
    name: String,
    updated_at: Option<String>,
    html_url: String,
}

#[derive(Deserialize)]
struct Commit {}

#[derive(Deserialize)]
#[serde(untagged)]
enum Payload {
    Package {
        package: Package,
    },
    Push {
        after: String,
        commits: Vec<Commit>,
        repository: Repository,
    },
}

pub async fn open_issue(log: String, services: Vec<String>, payload: String) {
    let parsed_payload = serde_json::from_str::<Payload>(&payload);

    let body = match parsed_payload {
        Ok(Payload::Package { package, .. }) => PostIssueBody {
            title: format!("Deployment failed for package {}", package.name),
            body: format!(
                "Deployment for {services} failed.\nTriggered by the publication of [{package}]({package_url}) at {date}.\n\nLogs: ```\n{log}\n```",
                services = if services.is_empty() {
                    "all services".to_owned()
                } else {
                    services.join(", ")
                },
                package = package.name,
                date = package.updated_at.unwrap_or("None".to_owned()),
                package_url = package.html_url
            ),
            assignees: config().github_assignees.clone()
        },
        Ok(Payload::Push {
            after,
            commits,
            repository,
            ..
        }) => {
            let services = if services.is_empty() {
                "all services".to_owned()
            } else {
                services.join(", ")
            };

           PostIssueBody {
            title: format!("Deployment failed for {services} ({})", &after.as_str()[0..6]),
            body: format!(
                "Deployment for {services} failed.\nTriggered by the push of {count} commits on {repo_url}. HEAD after the push is {after}.\n\nLogs:\n```\n{log}\n```\n",
                count = commits.len(),
                repo_url = repository.html_url
            ),
            assignees: config().github_assignees.clone()
        }},
        Err(e) => {
            tracing::error!("Invalid request payload: {}", e);
            return;
        }
    };

    let output = Command::new("curl")
        .arg("-X")
        .arg("POST")
        .arg("https://api.github.com/repos/clicepfl/infra/issues")
        .arg("-H")
        .arg(format!(
            "Authorization: Bearer {}",
            config().github_access_token
        ))
        .arg("-H")
        .arg("Accept: application/vnd.github+json")
        .arg("-d")
        .arg(serde_json::to_string(&body).unwrap())
        .output();

    match output {
        Ok(r) => {
            tracing::info!("Issue opened: {r:#?}")
        }
        Err(e) => tracing::error!("Could not open issue: {e:#?}"),
    };
}
