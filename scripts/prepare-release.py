#!/usr/bin/env python3
"""Prepare protected-main release PRs; tag only the approved main commit."""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import tomllib


def run(*args):
    return subprocess.check_output(args, text=True).strip()


def git(*args):
    return run("git", *args)


def version_tuple(version):
    if not re.fullmatch(r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)", version):
        raise ValueError("Version must be X.Y.Z without leading zeroes")
    return tuple(map(int, version.split(".")))


def open_pr(branch, title, body):
    repo = os.environ["GITHUB_REPOSITORY"]
    prs = json.loads(run("gh", "pr", "list", "--repo", repo, "--base", "main",
                         "--head", branch, "--state", "open", "--json", "url"))
    if prs:
        return prs[0]["url"]
    return run("gh", "pr", "create", "--repo", repo, "--base", "main",
               "--head", branch, "--title", title, "--body", body)


def push_pr(branch, paths, title, body):
    # Preserve an existing preparation branch, including reviewer changes.
    if git("ls-remote", "--heads", "origin", f"refs/heads/{branch}"):
        return open_pr(branch, title, body)
    git("checkout", "-b", branch)
    git("add", *paths)
    git("commit", "-m", title)
    git("push", "origin", f"HEAD:refs/heads/{branch}")
    return open_pr(branch, title, body)


def release_checks():
    run("bash", "scripts/release-checks.sh")


def prepare(version, formula=False):
    requested = version_tuple(version)
    if git("status", "--porcelain"):
        raise ValueError("Working tree must be clean")
    git("fetch", "origin", "main", "--tags")
    git("checkout", "--detach", "origin/main")
    main_commit = git("rev-parse", "HEAD")
    git("config", "user.name", "github-actions[bot]")
    git("config", "user.email", "github-actions[bot]@users.noreply.github.com")
    if formula:
        # The tag already exists: refuse a formula PR for a different package.
        manifest = tomllib.loads(git("show", f"v{version}:Cargo.toml"))
        if manifest["package"]["version"] != version:
            raise ValueError("Formula tag does not match package version")
        branch = f"release/homebrew-v{version}"
        title = f"chore(release): update Homebrew formula for v{version}"
        if git("ls-remote", "--heads", "origin", f"refs/heads/{branch}"):
            return "pr", open_pr(branch, title, f"Update the local formula for v{version}.")
        run("bash", "scripts/render-homebrew-formula.sh", version)
        if not git("diff", "--", "netspeed-cli.rb"):
            return "unchanged", "Formula is already current."
        return "pr", push_pr(branch, ["netspeed-cli.rb"], title,
                             f"Update the local formula for released v{version}.")

    tags = git("tag", "--list").splitlines()
    versions = [version_tuple(tag[1:]) for tag in tags
                if re.fullmatch(r"v(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)", tag)]
    if versions and requested <= max(versions):
        raise ValueError("Version must be greater than every existing release tag")
    published = run("cargo", "search", "netspeed-cli", "--limit", "1")
    if re.search(r'^netspeed-cli = "' + re.escape(version) + r'"', published, re.MULTILINE):
        raise ValueError("Version is already published on crates.io")
    current = tomllib.loads(Path("Cargo.toml").read_text())["package"]["version"]
    if requested < version_tuple(current):
        raise ValueError("Version must not be older than main's package version")
    if version == current:
        release_checks()
        if git("status", "--porcelain"):
            raise ValueError("Release checks changed tracked files; prepare a PR first")
        # Pin the approved commit; fail if main advanced while gates ran.
        remote_main = git("ls-remote", "origin", "refs/heads/main").split()[0]
        if remote_main != main_commit:
            raise ValueError("main advanced during release checks; rerun preparation")
        git("tag", "-a", f"v{version}", main_commit, "-m", f"Release v{version}")
        git("push", "origin", f"refs/tags/v{version}")
        return "tag", f"Tagged approved main commit {main_commit}. Publishing runs on the tag event."

    branch = f"release/netspeed-cli-v{version}"
    title = f"chore(release): bump to v{version}"
    body = f"Prepare v{version}. After approval and merge, rerun Release with version {version} to tag main."
    if git("ls-remote", "--heads", "origin", f"refs/heads/{branch}"):
        return "pr", open_pr(branch, title, body)
    path = Path("Cargo.toml")
    text = path.read_text()
    # Scope the edit to [package], never dependency versions.
    text = re.sub(r'(?ms)(^\[package\]\n.*?^version\s*=\s*)"[^"]+"',
                  lambda match: match[1] + f'"{version}"', text, count=1)
    if tomllib.loads(text)["package"]["version"] != version:
        raise ValueError("Could not update the package version")
    path.write_text(text)
    run("cargo", "check")
    run("cargo", "build", "--quiet")
    run("git-cliff", "--config", ".cliff.toml", "--tag", f"v{version}", "--output", "CHANGELOG.md")
    release_checks()
    return "pr", push_pr(branch, ["Cargo.toml", "Cargo.lock", "CHANGELOG.md", "completions", "netspeed-cli.1"],
                         title, body)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("version")
    parser.add_argument("--formula", action="store_true")
    args = parser.parse_args()
    try:
        action, message = prepare(args.version, args.formula)
        print(message)
        if os.environ.get("GITHUB_STEP_SUMMARY"):
            with open(os.environ["GITHUB_STEP_SUMMARY"], "a") as summary:
                summary.write(f"Release preparation: {action}\n\n{message}\n")
    except (ValueError, KeyError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"Release preparation failed: {error}\n")


if __name__ == "__main__":
    main()
