use crate::config::Service;
use std::process::Command;
use tracing::{span, Level};

fn try_run(command: Option<&String>, service: &str) -> bool {
    let Some(command) = command else {
        tracing::debug!("Command is empty, skipping");
        return true;
    };

    tracing::debug!(command);

    let res = Command::new("sh")
        .args(["-c", command])
        .env("SERVICE", service)
        .output();

    match res {
        Ok(output) if !output.status.success() => {
            tracing::error!("Failed with status {}", output.status,);
            tracing::error!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
            tracing::error!("STDOUT: {}", String::from_utf8_lossy(&output.stderr));
            false
        }
        Err(e) => {
            tracing::error!("Unable to spawn command: {e:#?}");
            false
        }
        _ => true,
    }
}

pub fn restart(name: &str, service: &Service, default: &Service) -> bool {
    let _enter = span!(Level::INFO, "service", name).entered();

    tracing::info!("Restarting...");

    let span = span!(Level::DEBUG, "stop_command").entered();
    if !try_run(
        service
            .stop_command
            .as_ref()
            .or(default.stop_command.as_ref()),
        name,
    ) {
        return false;
    };
    span.exit();

    let span = span!(Level::DEBUG, "pre_start_command").entered();
    if !try_run(
        service
            .pre_start_command
            .as_ref()
            .or(default.pre_start_command.as_ref()),
        name,
    ) {
        return false;
    };
    span.exit();

    let span = span!(Level::DEBUG, "start_command").entered();
    if !try_run(
        service
            .start_command
            .as_ref()
            .or(default.start_command.as_ref()),
        name,
    ) {
        return false;
    };
    span.exit();

    let span = span!(Level::DEBUG, "stop_command").entered();
    if !try_run(
        service
            .post_start_command
            .as_ref()
            .or(default.post_start_command.as_ref()),
        name,
    ) {
        return false;
    };
    span.exit();

    tracing::info!("Completed !");
    true
}
