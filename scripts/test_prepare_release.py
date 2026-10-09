import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

SCRIPT = Path(__file__).with_name("prepare-release.py").resolve()
spec = importlib.util.spec_from_file_location("prepare_release", SCRIPT)
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


class ReleasePreparationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.previous = os.getcwd()
        self.root = Path(self.temp.name)
        self.remote = self.root / "remote.git"
        subprocess.run(["git", "init", "--bare", str(self.remote)], check=True, capture_output=True)
        work = self.root / "work"
        work.mkdir()
        os.chdir(work)
        self.git("init", "-b", "main")
        self.git("config", "user.email", "test@example.invalid")
        self.git("config", "user.name", "Test")
        Path("Cargo.toml").write_text('[package]\nname = "netspeed-cli"\nversion = "1.2.3"\n\n[dependencies]\nversion = "9.0.0"\n')
        for name in ("Cargo.lock", "CHANGELOG.md", "netspeed-cli.1", "netspeed-cli.rb"):
            Path(name).write_text("fixture")
        Path("completions").mkdir()
        Path("completions/fixture").write_text("fixture")
        self.git("add", ".")
        self.git("commit", "-m", "fixture")
        self.git("remote", "add", "origin", str(self.remote))
        self.git("push", "origin", "main")
        self.git("tag", "v1.2.2")
        self.git("push", "origin", "v1.2.2")
        # Main rejects every update. Branch and tag writes remain possible.
        hook = self.remote / "hooks/pre-receive"
        hook.write_text('#!/bin/sh\nwhile read old new ref; do\n  [ "$ref" != refs/heads/main ] || exit 1\ndone\n')
        hook.chmod(0o755)
        self.main = self.git("rev-parse", "HEAD")
        self.calls = []
        self.prs = []
        self.check_failure = False
        self.advance_main = False
        self.dirty_checks = False
        self.env = patch.dict(os.environ, {"GITHUB_REPOSITORY": "test/repo"})
        self.env.start()
        self.runner = release.run
        self.mock = patch.object(release, "run", side_effect=self.mock_run)
        self.mock.start()

    def tearDown(self):
        self.mock.stop()
        self.env.stop()
        os.chdir(self.previous)
        self.temp.cleanup()

    def git(self, *args):
        return subprocess.check_output(["git", *args], text=True, stderr=subprocess.PIPE).strip()

    def mock_run(self, *args):
        self.calls.append(args)
        if args[0] == "git":
            return self.runner(*args)
        if args[:3] == ("gh", "pr", "list"):
            return json.dumps(self.prs)
        if args[:3] == ("gh", "pr", "create"):
            return "https://example.invalid/pr/1"
        if args[:2] == ("cargo", "search"):
            return 'netspeed-cli = "1.2.2"'
        if args[:2] == ("cargo", "check"):
            Path("Cargo.lock").write_text(Path("Cargo.toml").read_text())
        if args[0] == "git-cliff":
            Path("CHANGELOG.md").write_text("prepared release")
        if args[:2] == ("bash", "scripts/render-homebrew-formula.sh"):
            Path("netspeed-cli.rb").write_text(f"version {args[2]}")
        if args[:2] == ("bash", "scripts/release-checks.sh"):
            if self.check_failure:
                raise subprocess.CalledProcessError(1, args)
            if self.dirty_checks:
                Path("netspeed-cli.1").write_text("uncommitted generated docs")
            if self.advance_main:
                self.git("commit", "--allow-empty", "-m", "main advanced")
                subprocess.run(["git", "--git-dir", str(self.remote), "fetch", str(Path.cwd()),
                                "HEAD:refs/heads/main"], check=True, capture_output=True)
        return ""

    def assert_main_untouched(self):
        self.assertEqual(self.git("ls-remote", "origin", "refs/heads/main").split()[0], self.main)

    def test_version_change_opens_pr_without_tagging_protected_main(self):
        action, url = release.prepare("1.2.4")
        self.assertEqual(action, "pr")
        self.assertIn("/pr/", url)
        self.assert_main_untouched()
        self.assertEqual(self.git("ls-remote", "origin", "refs/tags/v1.2.4"), "")
        self.assertIn('version = "1.2.4"', self.git("show", "HEAD:Cargo.toml"))
        self.assertIn('version = "9.0.0"', self.git("show", "HEAD:Cargo.toml"))
        self.assertTrue(self.git("ls-remote", "origin", "refs/heads/release/netspeed-cli-v1.2.4"))

    def test_approved_main_is_tagged_without_main_push(self):
        self.assertEqual(release.prepare("1.2.3")[0], "tag")
        self.assert_main_untouched()
        self.assertEqual(self.git("rev-parse", "v1.2.3^{commit}"), self.main)
        self.assertTrue(self.git("ls-remote", "origin", "refs/tags/v1.2.3"))
        self.assertFalse(any(call[:3] == ("gh", "pr", "create") for call in self.calls))

    def test_existing_preparation_is_preserved_and_pr_reused(self):
        release.prepare("1.2.4")
        branch = "refs/heads/release/netspeed-cli-v1.2.4"
        original = self.git("ls-remote", "origin", branch)
        self.prs = [{"url": "https://example.invalid/existing"}]
        self.calls.clear()
        self.assertEqual(release.prepare("1.2.4"), ("pr", self.prs[0]["url"]))
        self.assertEqual(self.git("ls-remote", "origin", branch), original)
        self.assertFalse(any(call[0] == "git" and "push" in call for call in self.calls))

    def test_failed_gates_do_not_push_branch_or_tag(self):
        self.check_failure = True
        with self.assertRaises(subprocess.CalledProcessError):
            release.prepare("1.2.4")
        self.assert_main_untouched()
        self.assertEqual(self.git("ls-remote", "origin", "refs/heads/release/netspeed-cli-v1.2.4"), "")
        self.assertEqual(self.git("ls-remote", "origin", "refs/tags/v1.2.4"), "")

    def test_main_advancing_aborts_tagging(self):
        self.advance_main = True
        with self.assertRaisesRegex(ValueError, "main advanced"):
            release.prepare("1.2.3")
        self.assertEqual(self.git("ls-remote", "origin", "refs/tags/v1.2.3"), "")

    def test_existing_tag_and_older_version_are_rejected(self):
        with self.assertRaisesRegex(ValueError, "greater"):
            release.prepare("1.2.2")
        self.git("tag", "v1.2.3")
        self.git("push", "origin", "v1.2.3")
        with self.assertRaisesRegex(ValueError, "greater"):
            release.prepare("1.2.3")

    def test_formula_update_uses_pr_and_preserves_main(self):
        self.git("tag", "v1.2.3")
        self.git("push", "origin", "v1.2.3")
        self.assertEqual(release.prepare("1.2.3", formula=True)[0], "pr")
        self.assert_main_untouched()
        self.assertEqual(self.git("show", "HEAD:netspeed-cli.rb"), "version 1.2.3")
        self.assertTrue(self.git("ls-remote", "origin", "refs/heads/release/homebrew-v1.2.3"))

    def test_invalid_input_has_no_side_effects(self):
        for version in ("01.2.3", "1.2.3-extra", "v1.2.3", "$(touch bad)"):
            with self.subTest(version=version), self.assertRaises(ValueError):
                release.prepare(version)
        self.assertEqual(self.calls, [])

    def test_failed_tag_checks_never_create_a_tag(self):
        self.check_failure = True
        with self.assertRaises(subprocess.CalledProcessError):
            release.prepare("1.2.3")
        self.assertEqual(self.git("ls-remote", "origin", "refs/tags/v1.2.3"), "")
        self.assert_main_untouched()

    def test_uncommitted_generated_docs_prevent_tagging(self):
        self.dirty_checks = True
        with self.assertRaisesRegex(ValueError, "changed tracked files"):
            release.prepare("1.2.3")
        self.assertEqual(self.git("ls-remote", "origin", "refs/tags/v1.2.3"), "")

    def test_formula_rerun_reuses_pr_without_overwriting_branch(self):
        self.git("tag", "v1.2.3")
        self.git("push", "origin", "v1.2.3")
        release.prepare("1.2.3", formula=True)
        branch = "refs/heads/release/homebrew-v1.2.3"
        original = self.git("ls-remote", "origin", branch)
        self.prs = [{"url": "https://example.invalid/formula"}]
        self.calls.clear()
        self.assertEqual(release.prepare("1.2.3", formula=True), ("pr", self.prs[0]["url"]))
        self.assertEqual(self.git("ls-remote", "origin", branch), original)
        self.assertFalse(any(call[0] == "git" and "push" in call for call in self.calls))
