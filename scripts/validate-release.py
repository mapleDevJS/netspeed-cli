#!/usr/bin/env python3
"""Validate the checked-out release tag before building or publishing."""
import argparse
import re
import subprocess
import tomllib


def git(*args):
    return subprocess.check_output(["git", *args], text=True, stderr=subprocess.PIPE).strip()


def validate(tag, main_ref="origin/main"):
    if not re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+", tag):
        raise ValueError("Release tag must be vX.Y.Z")
    commit = git("rev-parse", f"{tag}^{{commit}}")
    if git("rev-parse", "HEAD") != commit:
        raise ValueError("HEAD does not match the release tag")
    version = tomllib.loads(git("show", f"{commit}:Cargo.toml"))["package"]["version"]
    if tag != f"v{version}":
        raise ValueError(f"Tag {tag} does not match Cargo package version {version}")
    subprocess.run(["git", "merge-base", "--is-ancestor", commit, main_ref], check=True, capture_output=True)
    return version


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("tag")
    args = parser.parse_args()
    try:
        print(f"Validated release {validate(args.tag)}")
    except (ValueError, KeyError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"Release validation failed: {error}\n")


if __name__ == "__main__":
    main()
