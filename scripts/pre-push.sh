#!/usr/bin/env bash
# Installed by `just install-hooks`. Validate the committed candidate before push.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
# Git exports repository paths to hooks. Fixtures must discover their own repos.
while IFS= read -r git_variable; do
  unset "$git_variable"
done < <(git rev-parse --local-env-vars)
candidate_sha="$(git rev-parse HEAD)"
has_updates=false
while read -r _local_ref local_sha _remote_ref _remote_sha; do
  [[ "$local_sha" =~ ^0+$ ]] && continue
  if [[ "$(git rev-parse "$local_sha^{commit}")" != "$candidate_sha" ]]; then
    echo 'Push only the checked-out commit; switch branches before pushing another candidate.' >&2
    exit 1
  fi
  has_updates=true
done
[[ "$has_updates" == false ]] && exit 0
if [[ -n "$(git status --porcelain --untracked-files=normal)" ]]; then
  echo 'Commit or stash changes before pushing: local CI must check the committed candidate.' >&2
  exit 1
fi
just ci-local
if [[ "$(git rev-parse HEAD)" != "$candidate_sha" || -n "$(git status --porcelain --untracked-files=normal)" ]]; then
  echo 'Candidate changed during local CI; rerun the push.' >&2
  exit 1
fi
