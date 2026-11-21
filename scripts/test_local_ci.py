"""Exercise the pre-push candidate boundary using disposable Git repositories."""
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest


HOOK = Path(__file__).with_name("pre-push.sh").resolve()


class WorkflowTriggerPolicyTests(unittest.TestCase):
    def test_only_release_tags_can_start_repository_workflows_automatically(self):
        workflows = HOOK.parent.parent / ".github" / "workflows"
        for path in workflows.glob("*.yml"):
            with self.subTest(workflow=path.name):
                block = path.read_text().split("\non:\n", 1)[1].split("\n\n", 1)[0]
                triggers = set(re.findall(r"^  ([a-z_]+):", block, re.MULTILINE))
                expected = {"workflow_dispatch", "push"} if path.name == "release.yml" else {"workflow_dispatch"}
                self.assertEqual(triggers, expected)
                if path.name == "release.yml":
                    self.assertIn("    tags:\n      - 'v*'", block)
                    self.assertNotIn("branches:", block)


class LocalCiHookTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        root = Path(self.temp.name)
        self.repo = root / "repo"
        self.repo.mkdir()
        bin_dir = root / "bin"
        bin_dir.mkdir()
        self.log = root / "gate.log"
        self.env = dict(os.environ, PATH=f"{bin_dir}{os.pathsep}{os.environ['PATH']}",
                        GATE_LOG=str(self.log), GIT_AUTHOR_NAME="CI test",
                        GIT_AUTHOR_EMAIL="ci@example.invalid",
                        GIT_COMMITTER_NAME="CI test",
                        GIT_COMMITTER_EMAIL="ci@example.invalid")
        just = bin_dir / "just"
        just.write_text('#!/bin/sh\necho "$*" >> "$GATE_LOG"\n'
                        'if [ "${CHANGE_CANDIDATE:-0}" = 1 ]; then echo changed >> fixture; fi\n'
                        'exit "${GATE_EXIT:-0}"\n')
        just.chmod(0o755)
        self.git("init", "-q")
        (self.repo / "fixture").write_text("first\n")
        self.git("add", "fixture")
        self.git("commit", "-qm", "fixture")
        self.sha = self.git("rev-parse", "HEAD")

    def git(self, *args):
        return subprocess.check_output(["git", *args], cwd=self.repo,
                                       env=self.env, text=True).strip()

    def hook(self, sha=None):
        update = f"refs/heads/main {sha or self.sha} refs/heads/main {'0' * 40}\n"
        return subprocess.run(["bash", str(HOOK)], cwd=self.repo, env=self.env,
                              input=update, text=True, capture_output=True)

    def test_clean_candidate_runs_complete_gate(self):
        result = self.hook()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.log.read_text(), "ci-local\n")

    def test_dirty_candidate_never_runs_gate(self):
        (self.repo / "fixture").write_text("dirty\n")
        self.assertNotEqual(self.hook().returncode, 0)
        self.assertFalse(self.log.exists())

    def test_other_commit_is_rejected(self):
        (self.repo / "fixture").write_text("second\n")
        self.git("commit", "-qam", "second")
        self.assertNotEqual(self.hook(self.sha).returncode, 0)
        self.assertFalse(self.log.exists())

    def test_deletion_needs_no_gate(self):
        self.assertEqual(self.hook("0" * 40).returncode, 0)
        self.assertFalse(self.log.exists())

    def test_failed_gate_blocks_push(self):
        self.env["GATE_EXIT"] = "7"
        self.assertEqual(self.hook().returncode, 7)

    def test_candidate_changed_during_gate_is_rejected(self):
        self.env["CHANGE_CANDIDATE"] = "1"
        self.assertNotEqual(self.hook().returncode, 0)


if __name__ == "__main__":
    unittest.main()
