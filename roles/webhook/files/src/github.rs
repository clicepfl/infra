//! This crate provides function and types to interact with GitHub API, both through HTTPS calls or an incoming webhook.

use actix_web::http::header::HeaderMap;
use reqwest::{Client, Method};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    config,
    github::{
        event::{parse_payload, Action, Payload, Push},
        issues::{
            EmptyBody, IssueCommentBody, OpenIssueBody, PostIssueBody, UpdateIssueBody,
        },
    },
};

/// Webhook data types.
pub mod event;
/// Data types for the Issue API paths.
mod issues;

/// Util function to call the GitHub API.
async fn github_api_call<B, R>(uri: &str, method: Method, body: B) -> Result<R, std::io::Error>
where
    B: Serialize,
    R: DeserializeOwned,
{
    let client = Client::new();

    let response = client
        .request(method, uri)
        .bearer_auth(&config().github_access_token)
        .header("Accept", "application/vnd.github+json")
        .body(serde_json::to_vec(&body)?)
        .send()
        .await
        .map_err(std::io::Error::other)?;

    let body = response
        .text()
        .await
        .map_err(std::io::Error::other)?;

    serde_json::from_str(&body).map_err(std::io::Error::other)
}

/// Open an issue on the infra repository using the provided metadata.
///
/// - `log`: The log produced by handling the event (see [log][crate::log]).
/// - `services`: The services that were targeted for redeployment.
/// - `headers` and `payload`: Data provided by GitHub through the webhook.
pub async fn open_issue(log: String, service: Option<&str>, headers: &HeaderMap, payload: &[u8]) {
    let parsed_payload = parse_payload(headers, payload);

    let body = match parsed_payload {
        Ok(Payload::Action(Action::Published { package }) )=> PostIssueBody {
            title: format!("Deployment failed for package {}", package.name),
            body: format!(
                "Deployment for {service} failed.\nTriggered by the publication of [{package}]({package_url}) at {date}.\n\nLogs: ```\n{log}\n```",
                service = service.unwrap_or("all services"),
                package = package.name,
                date = package.updated_at.unwrap_or("None".to_owned()),
                package_url = package.html_url
            ),
            assignees: config().github_assignees.clone()
        },
        Ok(Payload::Push( Push{
            after,
            commits,
            repository,
            ..
        })) => {
            let service = service.unwrap_or("all services");

           PostIssueBody {
            title: format!("Deployment failed for {service} ({})", &after.as_str()[0..6]),
            body: format!(
                "Deployment for {service} failed.\nTriggered by the push of {count} commits on {repo_url}. HEAD after the push is {after}.\n\nLogs:\n```\n{log}\n```\n",
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
        Method::POST,
        body,
    )
    .await
    {
        Ok(_) => tracing::info!("Issue opened"),
        Err(e) => tracing::error!("Could not open issue: {e:#?}"),
    };
}

/// Closes all issues referencing the failed deployment of `service`, or all of them if `service` is `None`.
///
/// - `headers` and `payload`: Data provided by GitHub through the webhook.
pub async fn close_issues(service: Option<&str>, headers: &HeaderMap, payload: &[u8]) {
    let fix_source = match parse_payload(headers, payload) {
        Ok(Payload::Action(Action::Published { package })) => {
            format!(
                "package {} ({})",
                package.name,
                package.updated_at.unwrap_or_default()
            )
        }
        Ok(Payload::Push(Push { after, .. })) => format!("commit {}", &after.as_str()[0..6]),
        Err(_) => "<unable to parse hook payload>".to_owned(),
    };

    let issues: Vec<OpenIssueBody> = match github_api_call(
        "https://api.github.com/repos/clicepfl/infra/issues",
        Method::GET,
        EmptyBody {},
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
            i.title.starts_with("Deployment failed") && service.is_none_or(|s| i.title.contains(s))
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
            Method::POST,
            IssueCommentBody {
                body: format!("Fixed by {}", fix_source),
            },
        )
        .await;

        if result.is_ok() {
            result = github_api_call(
                &issue_url,
                Method::PATCH,
                UpdateIssueBody {
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
