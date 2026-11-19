#!/usr/bin/env python3
"""Require a successful release-validation run for the exact tagged commit."""
import argparse
import json
import re
import subprocess


def run(*args):
    return subprocess.check_output(args, text=True, stderr=subprocess.PIPE).strip()


def validation_run_id(tag_contents):
    matches = re.findall(r"^Release validation run: ([0-9]+)$", tag_contents, re.MULTILINE)
    if len(matches) != 1:
        raise ValueError("Tag must reference exactly one release validation run")
    return matches[0]


def validate_run(run_data, tag_sha):
    expected = {
        "event": "workflow_dispatch",
        "headSha": tag_sha,
        "conclusion": "success",
        "workflowName": "Release",
    }
    for key, value in expected.items():
        if run_data.get(key) != value:
            raise ValueError(f"Release validation run has unexpected {key}")


def verify(tag):
    tag_contents = run("git", "for-each-ref", "--format=%(contents)", f"refs/tags/{tag}")
    run_id = validation_run_id(tag_contents)
    subprocess.run(
        ["gh", "run", "watch", run_id, "--exit-status", "--interval", "10"],
        check=True,
    )
    run_data = json.loads(run(
        "gh", "run", "view", run_id,
        "--json", "event,headSha,conclusion,workflowName",
    ))
    tag_sha = run("git", "rev-parse", f"{tag}^{{commit}}")
    validate_run(run_data, tag_sha)
    print(f"Verified release validation run {run_id} for {tag} ({tag_sha})")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("tag")
    args = parser.parse_args()
    try:
        verify(args.tag)
    except (ValueError, json.JSONDecodeError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"Release validation evidence failed: {error}\n")


if __name__ == "__main__":
    main()
