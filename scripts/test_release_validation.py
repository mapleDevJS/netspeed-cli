import importlib.util
from pathlib import Path
import unittest

SCRIPT = Path(__file__).with_name("verify-release-validation.py").resolve()
spec = importlib.util.spec_from_file_location("verify_release_validation", SCRIPT)
validation = importlib.util.module_from_spec(spec)
spec.loader.exec_module(validation)


class ReleaseValidationEvidenceTests(unittest.TestCase):
    def test_extracts_run_id_from_annotated_tag(self):
        contents = "Release v1.2.3\n\nRelease validation run: 123456"
        self.assertEqual(validation.validation_run_id(contents), "123456")

    def test_rejects_missing_or_ambiguous_run_id(self):
        for contents in (
            "Release v1.2.3",
            "Release validation run: 1\nRelease validation run: 2",
            "Release validation run: abc",
        ):
            with self.subTest(contents=contents):
                with self.assertRaisesRegex(ValueError, "exactly one"):
                    validation.validation_run_id(contents)

    def test_accepts_successful_run_for_the_tagged_commit(self):
        validation.validate_run({
            "event": "workflow_dispatch",
            "headSha": "abc123",
            "conclusion": "success",
            "workflowName": "Release",
        }, "abc123")

    def test_rejects_non_release_run_or_different_commit(self):
        valid = {
            "event": "workflow_dispatch",
            "headSha": "abc123",
            "conclusion": "success",
            "workflowName": "Release",
        }
        for key, value in (("event", "push"), ("headSha", "def456"),
                           ("conclusion", "failure"), ("workflowName", "CI")):
            with self.subTest(key=key):
                run_data = {**valid, key: value}
                with self.assertRaisesRegex(ValueError, f"unexpected {key}"):
                    validation.validate_run(run_data, "abc123")


if __name__ == "__main__":
    unittest.main()
