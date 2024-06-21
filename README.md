# Clic Server configuration

This repository contains the complete configuration of the Clic's server. It uses [Ansible](https://docs.ansible.com/ansible/latest/index.html) scripts for automated installation and deployment.

## Playbooks

There are two playbooks, at the root of the repository:

- [`init.yaml`](init.yaml): Initial set up of the server, must be run after each the machine has been reinstalled. It is responsible for installing the minimal dependencies (e.g. ansible, borgbackup, docker, etc.) as well as the required ansible-roles and automatic updates.

- [`deploy.yaml`](deploy.yaml): (Re)deploy one or several services, and setups/updates the automatic backups. It relies on extra variables to get the services' secrets or some configuration data (like the Unix user, backup repository, etc.). Ideally, it should be run through [`ansible-pull`](https://docs.ansible.com/ansible/latest/cli/ansible-pull.html):

  ```sh
  ansible-pull \
      -U git@github.com:clicepfl/clic-infra.git \ # URL of the infra repo
      -e @/var/secrets.yaml \ # File containing all the secrets
      deploy.yaml \
      --extra-vars SERVICE=keycloak # Optional. Services to (re)deploy
  ```

  The `/var/secrets.yaml` file, based on [`secrets.yaml.example`](./secrets.yaml.example), contains the configuration options and secrets (like database/admin passwords). The playbook will check beforehand that all the variables are present.

  The optional `SERVICE` variable is a comma-separated list of service to (re)deploy. Useful for partial updates, e.g. when calling from the webhook.

## Services

Each service has a dedicated role for deployment. It is called by the `deploy.yaml` playbook, and is given as parameter `service` the corresponding entry in the `services` dictionnary from the secrets file.
**Important:** when using files/templates from the respective directories, it may be required to append the `role_path` variable at the start of the path (e.g. `"{{ role_path }}/files/docker-compose.yaml"`).

If a service's role needs to generate a configuration file, it needs to be stored in the directory `{{ general.config_dir }}/{{ SERVICE }}` (`general.config_dir` is specified in the `secrets.yaml` file). It then must be mounted into the container using a [bind mount](https://docs.docker.com/storage/bind-mounts/). Since bind mounts do not allow to change the permissions/ownership of the file, the role must take care of setting those properly.

### Caddy

Caddy is the reverse-proxy used to dispatch incoming HTTP requests to the different services. It also handles HTTPS with the client. It communicates with the services through HTTP, to avoid having to manage local certificates. Since it needs to handle connections on both IPv4 and IPv6, it needs to run on bare-metal, as docker swarm does not yet allow to bind to an IPv6 address.

### Webhook

The webhook service is used to trigger (partial) re-deployment of the infrastructure. Unlike the other services, the webhook runs on bare metal, to be able to do all necessary modifications of the server.

It is designed to receive packages from GitHub, in order to automatically re-deploy services when they are updated. See the [official documentation](https://docs.github.com/en/webhooks/about-webhooks) and the dedicated [README](roles/webhook/files/README.md).
