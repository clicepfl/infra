use std::process::Command;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

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

#[derive(Serialize, Deserialize)]
struct EmptyBody {}

async fn github_api_call<B, R>(uri: &str, method: &str, body: B) -> Result<R, std::io::Error>
where
    B: Serialize,
    R: DeserializeOwned,
{
    Command::new("curl")
        .arg("-X")
        .arg(method)
        .arg(uri)
        .arg("-H")
        .arg(format!(
            "Authorization: Bearer {}",
            config().github_access_token
        ))
        .arg("-H")
        .arg("Accept: application/vnd.github+json")
        .arg("-d")
        .arg(serde_json::to_string(&body).unwrap())
        .output()
        .map(|o| {
            tracing::trace!(
                "Got github response: {}",
                String::from_utf8_lossy(&o.stdout)
            );
            serde_json::from_slice(&o.stdout).unwrap()
        })
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

    match github_api_call::<_, EmptyBody>(
        "https://api.github.com/repos/clicepfl/infra/issues",
        "POST",
        body,
    )
    .await
    {
        Ok(_) => tracing::info!("Issue opened"),
        Err(e) => tracing::error!("Could not open issue: {e:#?}"),
    };
}

pub async fn close_issues(services: Vec<String>, payload: String) {
    let fix_source = match serde_json::from_str::<Payload>(&payload) {
        Ok(Payload::Package { package }) => {
            format!(
                "package {} ({})",
                package.name,
                package.updated_at.unwrap_or_default()
            )
        }
        Ok(Payload::Push { after, .. }) => format!("commit {}", &after.as_str()[0..6]),
        Err(_) => "<unable to parse hook payload>".to_owned(),
    };

    #[derive(Deserialize, Clone)]
    struct OpenIssue {
        number: u32,
        title: String,
    }

    #[derive(Serialize)]
    struct ListIssuesRequestBody {}

    #[derive(Serialize)]
    struct IssueCommentRequestBody {
        body: String,
    }

    #[derive(Serialize)]
    struct UpdateIssueRequestBody {
        state: String,
    }

    let issues: Vec<OpenIssue> = match github_api_call(
        "https://api.github.com/repos/clicepfl/infra/issues",
        "GET",
        ListIssuesRequestBody {},
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Could not list repo issues: {e:#?}");
            return;
        }
    };

    let matching_issues = issues
        .into_iter()
        .filter(|i| {
            i.title.starts_with("Deployment failed") && services.iter().any(|s| i.title.contains(s))
        })
        .collect::<Vec<_>>();

    tracing::info!(
        "Closing issues {}",
        matching_issues
            .iter()
            .map(|i| i.number.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );

    for issue in matching_issues.iter() {
        let issue_url = format!(
            "https://api.github.com/repos/clicepfl/infra/issues/{}",
            issue.number
        );
        let issue_comment_url = format!("{}/comments", issue_url);

        let mut result: Result<EmptyBody, _> = github_api_call(
            &issue_comment_url,
            "POST",
            IssueCommentRequestBody {
                body: format!("Fixed by {}", fix_source),
            },
        )
        .await;

        if result.is_ok() {
            result = github_api_call(
                &issue_url,
                "PATCH",
                UpdateIssueRequestBody {
                    state: "closed".to_owned(),
                },
            )
            .await;
        }

        if let Err(e) = result {
            tracing::error!("Could not update issue {}: {e:#?}", issue.number);
        }
    }
}
