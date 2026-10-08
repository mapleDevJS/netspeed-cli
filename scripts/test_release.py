import importlib.util
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("validate-release.py").resolve()
spec = importlib.util.spec_from_file_location("validate_release", SCRIPT)
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


class ReleaseValidationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.previous = os.getcwd()
        os.chdir(self.temp.name)
        self.git("init", "-b", "main")
        self.git("config", "user.email", "test@example.invalid")
        self.git("config", "user.name", "Test")
        Path("Cargo.toml").write_text('[package]\nname = "test"\nversion = "1.2.3"\n')
        self.git("add", "Cargo.toml")
        self.git("commit", "-m", "fixture")
        self.git("update-ref", "refs/remotes/origin/main", "HEAD")
        self.git("tag", "v1.2.3")

    def tearDown(self):
        os.chdir(self.previous)
        self.temp.cleanup()

    def git(self, *args):
        subprocess.run(["git", *args], check=True, capture_output=True)

    def test_matching_version(self):
        self.assertEqual(release.validate("v1.2.3"), "1.2.3")

    def test_mismatched_version(self):
        self.git("tag", "v1.2.4")
        with self.assertRaisesRegex(ValueError, "does not match Cargo"):
            release.validate("v1.2.4")

    def test_wrong_checkout(self):
        Path("other").write_text("fixture")
        self.git("add", "other")
        self.git("commit", "-m", "other")
        with self.assertRaisesRegex(ValueError, "HEAD does not match"):
            release.validate("v1.2.3")

    def test_off_main(self):
        self.git("checkout", "--orphan", "other")
        self.git("commit", "-m", "unrelated")
        self.git("tag", "-f", "v1.2.3")
        with self.assertRaises(subprocess.CalledProcessError):
            release.validate("v1.2.3")

    def test_malformed_tag(self):
        with self.assertRaisesRegex(ValueError, "vX.Y.Z"):
            release.validate("v1.2.3-extra")
