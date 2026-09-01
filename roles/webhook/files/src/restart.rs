//! Functions to redeploy a service.

use crate::config::Service;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tracing::{span, Level};

/// Attempts to run a shell command, logging its output in case of failure.
///
/// - `command`: The command to run. This function does nothing if this value is `None`.
/// - `service`: The value to pass to the command through the `SERVICE` environment variable.
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

/// Restarts the given service.
///
/// - `name`: Name of the service to restart
/// - `service`: Specific deployment configuration for that service.
/// - `default`: Default deployment configuration.
///
/// Holds `lock_state` for the full restart sequence so concurrent webhook
/// deliveries cannot run overlapping ansible-pull deploys (#193).
pub fn restart(name: &str, service: &Service, default: &Service, lock_state: Arc<Mutex<()>>) -> bool {
    let _deploy_guard = lock_state.lock().unwrap_or_else(|e| e.into_inner());
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