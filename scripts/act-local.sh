#!/usr/bin/env bash
# Respect an explicit host; otherwise use Docker's selected context, including Colima.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
if [[ -z "${DOCKER_HOST:-}" ]]; then
  DOCKER_HOST="$(docker context inspect --format '{{.Endpoints.docker.Host}}')"
  export DOCKER_HOST
fi
container_args=()
if [[ -f .git ]]; then
  # act copies the gitfile, but its target lives outside a linked worktree.
  git_common_dir="$(git rev-parse --path-format=absolute --git-common-dir)"
  container_args=(--container-options "--volume \"$git_common_dir:$git_common_dir:ro\"")
fi
exec act --use-new-action-cache --no-cache-server --container-daemon-socket /var/run/docker.sock "${container_args[@]}" "$@"
