"""Exercise the pre-push candidate boundary using disposable Git repositories."""
import os
from pathlib import Path
import re
import shutil
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
        git_variables = subprocess.check_output(
            ["git", "rev-parse", "--local-env-vars"], text=True).splitlines()
        fixture_env = {key: value for key, value in os.environ.items()
                       if key not in git_variables}
        self.env = dict(fixture_env, PATH=f"{bin_dir}{os.pathsep}{os.environ['PATH']}",
                        GATE_LOG=str(self.log), GIT_AUTHOR_NAME="CI test",
                        GIT_AUTHOR_EMAIL="ci@example.invalid",
                        GIT_COMMITTER_NAME="CI test",
                        GIT_COMMITTER_EMAIL="ci@example.invalid")
        just = bin_dir / "just"
        just.write_text('#!/bin/sh\necho "$*" >> "$GATE_LOG"\n'
                        'if [ -n "${GIT_DIR:-}${GIT_WORK_TREE:-}${GIT_INDEX_FILE:-}${GIT_CONFIG_COUNT:-}" ]; then exit 42; fi\n'
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

    def test_hook_clears_repository_environment(self):
        self.env.update(GIT_DIR=str(self.repo / ".git"),
                        GIT_WORK_TREE=str(self.repo),
                        GIT_INDEX_FILE=str(self.repo / ".git" / "index"),
                        GIT_CONFIG_COUNT="0")
        result = self.hook()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.log.read_text(), "ci-local\n")

    def test_actual_push_keeps_fixture_git_context_out_of_gate(self):
        remote = Path(self.temp.name) / "remote.git"
        self.git("init", "--bare", "-q", str(remote))
        hook_path = self.repo / ".git" / "hooks" / "pre-push"
        shutil.copyfile(HOOK, hook_path)
        hook_path.chmod(0o755)
        self.git("push", "--quiet", str(remote), "HEAD:refs/heads/main")
        self.assertEqual(self.log.read_text(), "ci-local\n")
        self.assertEqual(self.git("rev-parse", "HEAD"), self.sha)
        self.assertEqual(self.git("status", "--porcelain"), "")

    def run_act_wrapper(self, checkout):
        script = checkout / "scripts" / "act-local.sh"
        script.parent.mkdir()
        shutil.copyfile(HOOK.with_name("act-local.sh"), script)
        args_file = Path(self.temp.name) / "act-args"
        act = Path(self.env["PATH"].split(os.pathsep)[0]) / "act"
        act.write_text('#!/bin/sh\nprintf "%s\\n" "$DOCKER_HOST" "$@" > "$ACT_ARGS"\n')
        act.chmod(0o755)
        env = dict(self.env, DOCKER_HOST="unix:///selected-context.sock",
                   ACT_ARGS=str(args_file))
        subprocess.run(["bash", str(script), "workflow_dispatch"],
                       env=env, check=True, capture_output=True)
        return args_file.read_text().splitlines()

    def test_normal_clone_uses_selected_host_without_extra_metadata_mount(self):
        args = self.run_act_wrapper(self.repo)
        self.assertEqual(args[0], "unix:///selected-context.sock")
        self.assertNotIn("--container-options", args)

    def test_linked_worktree_mounts_common_metadata_read_only(self):
        checkout = Path(self.temp.name) / "linked"
        self.git("worktree", "add", "--quiet", "--detach", str(checkout), "HEAD")
        args = self.run_act_wrapper(checkout)
        common_dir = (self.repo / ".git").resolve()
        mount = args[args.index("--container-options") + 1]
        self.assertEqual(mount, f'--volume "{common_dir}:{common_dir}:ro"')


if __name__ == "__main__":
    unittest.main()
