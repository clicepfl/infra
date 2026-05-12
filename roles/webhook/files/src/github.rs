//! This crate provides function and types to interact with GitHub API, both through HTTPS calls or an incoming webhook.

use actix_web::http::header::HeaderMap;
use reqwest::{Client, Method};
use serde::{de::DeserializeOwned, Serialize};
use regex::Regex;

use crate::{
    config,
    github::{
        event::{parse_payload, PackageAction, Payload, Push},
        issues::{EmptyBody, IssueCommentBody, OpenIssueBody, PostIssueBody, UpdateIssueBody},
    },
};

/// Webhook data types.
pub mod event;
/// Data types for the Issue API paths.
mod issues;

/// Util function to call the GitHub API.
async fn github_api_call<B, R>(
    uri: &str,
    method: Method,
    body: Option<B>,
) -> Result<R, std::io::Error>
where
    B: Serialize,
    R: DeserializeOwned,
{
    let client = Client::new();

    let mut request = client
        .request(method, uri)
        .bearer_auth(&config().github_access_token)
        .header("Accept", "application/vnd.github+json")
        // https://docs.github.com/en/rest/about-the-rest-api/api-versions?apiVersion=2022-11-28
        .header("X-GitHub-Api-Version", "2022-11-28")
        // https://docs.github.com/en/rest/using-the-rest-api/getting-started-with-the-rest-api?apiVersion=2022-11-28#user-agent
        .header("User-Agent", "CLIC-Webhook");

    if let Some(body) = body {
        request = request.body(serde_json::to_vec(&body)?)
    };

    request
        .send()
        .await
        .map_err(std::io::Error::other)?
        .json()
        .await
        .map_err(std::io::Error::other)
}

/// Open an issue on the infra repository using the provided metadata.
///
/// - `log`: The log produced by handling the event (see [log][crate::log]).
/// - `services`: The services that were targeted for redeployment.
/// - `headers` and `payload`: Data provided by GitHub through the webhook.
pub async fn open_issue(log: String, service: Option<&str>, headers: &HeaderMap, payload: &[u8]) {
    let parsed_payload = parse_payload(headers, payload);

    let body = match parsed_payload {
        Ok(Payload::Package(PackageAction::Published { package, repository }) )=> PostIssueBody {
            title: format!("Deployment failed for package from '{}'", repository.full_name),
            body: format!(
                "Deployment for {service} failed.\nTriggered by the publication of [{package}]({package_url}) at {date}.\n\nLogs:\n```\n{log}\n```\n",
                service = service.unwrap_or("all services"),
                package = package.name,
                date = package.updated_at.unwrap_or("None".to_owned()),
                package_url = package.html_url
            ),
            assignees: config().github_assignees.clone(),
            labels: vec![String::from("build failed")]
        },
        Ok(Payload::Push( Push{
            after,
            commits,
            repository,
            ..
        })) => {
            let service = service.unwrap_or("all services");

           PostIssueBody {
            title: format!("Deployment failed for {service} ({}) from '{}'", &after.as_str()[0..6], repository.full_name),
            body: format!(
                "Deployment for {service} failed.\nTriggered by the push of {count} commits on {repo_url}. HEAD after the push is {after}.\n\nLogs:\n```\n{log}\n```\n",
                count = commits.len(),
                repo_url = repository.html_url
            ),
            assignees: config().github_assignees.clone(),
            labels: vec![String::from("build failed")]
        }},
        Err(e) => {
            tracing::error!("Invalid request payload: {}", e);
            return;
        }
    };

    match github_api_call::<_, EmptyBody>(
        "https://api.github.com/repos/clicepfl/infra/issues",
        Method::POST,
        Some(body),
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
    let (fix_source, fix_repo_name) = match parse_payload(headers, payload) {
        Ok(Payload::Package(PackageAction::Published { package, repository })) => {(
            format!(
                "package {} ({})",
                package.name,
                package.updated_at.unwrap_or_default()
            ),
            repository.full_name
        )}
        Ok(Payload::Push(Push { after, repository, .. })) => (
            format!("commit {}", &after.as_str()[0..6]),
            repository.full_name
        ),
        Err(_) => ("<unable to parse hook payload>".to_owned(), "".to_owned()),
    };

    let issues: Vec<OpenIssueBody> = match github_api_call(
        "https://api.github.com/repos/clicepfl/infra/issues",
        Method::GET,
        Option::<EmptyBody>::None,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Could not list repo issues: {e:#?}");
            return;
        }
    };

    // Extract user and repo from anything containing " from 'user/repo'"
    let issue_rx = Regex::new(r".* from '([^'\/]*)\/([^'\/]*)'.*").unwrap();
    let matching_issues = issues
        .into_iter()
        .filter(|i| {
            i.labels.iter().any(|l| l == "build failed") && service.is_none_or(|s| {
                // Delete only if the repo from which the fix comes from is the one that caused the
                // issue, or assume 'clicepfl' if it was triggered manually
                // NOTE: This disallows deleting issues on manual restarts for services hosted
                // outside of clic's github but thats a difficult edge-case
                let Some(captures) = issue_rx.captures(&i.title) else { return false; };

                let Some(user) = captures.get(1) else { return false; };
                let Some(repo) = captures.get(2) else { return false; };

                if fix_repo_name == "" { // Triggered manually
                   user.as_str() == "clicepfl" && repo.as_str() == s
                } else {
                   format!("{}/{}", user.as_str(), repo.as_str()) == fix_repo_name
                }
            })
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
            Some(IssueCommentBody {
                body: format!("Fixed by {}", fix_source),
            }),
        )
        .await;

        if result.is_ok() {
            result = github_api_call(
                &issue_url,
                Method::PATCH,
                Some(UpdateIssueBody {
                    state: "closed".to_owned(),
                }),
            )
            .await;
        }

        if let Err(e) = result {
            tracing::error!("Could not update issue {}: {e:#?}", issue.number);
        }
    }
}
