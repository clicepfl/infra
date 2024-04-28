# Webhook

This folder contains a webhook to automatically redeploy the infrastructure - either entierly of partially - upon reception of a valid payload from GitHub. It is a simple webserver, communicating over HTTP.

There are two routes:
- `/`: redeploy the whole infrastructure.
- `/<service>`: redeploy a single service.

## Configuration

The configuration of the secrets and commands to run for deployment is done through a `config.json` file. By default, it uses the one at `/etc/webhook/config.json`, but this behavior can be changed by setting the `CONFIG_PATH` environment variable.

The schema is as follow:

```jsonc
{
    /// Secret used to sign the payload of the request. Also set on GitHub, to prevent malicious calls.
    "secret": "hmac-secret",

    /// Default command used to restart a service. Currently does not support any parameters.
    "generic_start_command": "echo generic >> /tmp/webhook.out",

    /// Map describing commands for each service.
    "services": {
        "service1": {
            "pre_start_command": "echo pre_start >> /tmp/webhook.out",
            "start_command": "echo start >> /tmp/webhook.out",
            "stop_command": "echo  stop >> /tmp/webhook.out"
        }
    }
}
```

The commands specified per services follow this order:
- `stop_command`
- `pre_start_command`
- `start_command`
- `generic_start_command`, only for global restart of if no `start_command` is set for the service.
- `post_start_command`
