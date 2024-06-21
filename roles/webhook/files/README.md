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

  /// Default command used to restart a service. The name of the service is provided in the environment variable `SERVICE`.
  "default": {
    "start_command": "echo $SERVICE >> /tmp/webhook.out"
  },

  /// Map describing commands for each service.
  "services": {
    "service1": {
      "pre_start_command": "echo pre_start >> /tmp/webhook.out",
      "start_command": "echo start >> /tmp/webhook.out",
      "stop_command": "echo  stop >> /tmp/webhook.out"
    },
    "service2": {}
  }
}
```

The commands specified per services follow this order:

1. `stop_command`
1. `pre_start_command`
1. `start_command`
1. `post_start_command`

Note that:

- Each service must be present in the `services` map, even if it only uses the default commands.
- If a command is specified neither in the service's object nor in the `default` one, nothing is ran.
