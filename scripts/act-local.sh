#!/usr/bin/env bash
# Respect an explicit host; otherwise use Docker's selected context, including Colima.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
if [[ -z "${DOCKER_HOST:-}" ]]; then
  DOCKER_HOST="$(docker context inspect --format '{{.Endpoints.docker.Host}}')"
  export DOCKER_HOST
fi
exec act --use-new-action-cache --no-cache-server --container-daemon-socket /var/run/docker.sock "$@"
