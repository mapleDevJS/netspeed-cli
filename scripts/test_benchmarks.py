import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("check-benchmarks.py")
spec = importlib.util.spec_from_file_location("benchmarks", SCRIPT)
benchmarks = importlib.util.module_from_spec(spec)
spec.loader.exec_module(benchmarks)


class BenchmarkTests(unittest.TestCase):
    def test_significant_slowdown_fails(self):
        self.assertTrue(
            benchmarks.regressions({"a": (100, 99, 101)}, {"a": (200, 199, 201)}, 0.1)
        )

    def test_small_absolute_slowdown_is_treated_as_runner_noise(self):
        baseline = {"validation/is_valid_ipv4/too_many": (92.255, 92.072, 92.425)}
        current = {"validation/is_valid_ipv4/too_many": (102.69, 102.28, 103.13)}
        self.assertFalse(benchmarks.regressions(baseline, current, 0.1))

    def test_noise_does_not_fail(self):
        self.assertFalse(
            benchmarks.regressions({"a": (100, 80, 120)}, {"a": (125, 100, 150)}, 0.1)
        )

    def test_missing_benchmark_fails(self):
        with self.assertRaises(ValueError):
            benchmarks.regressions({"a": (100, 99, 101)}, {}, 0.1)

    def test_ignored_url_microbenchmarks_do_not_block(self):
        baseline = {
            "url_build_test_url/file_2": (50, 49, 51),
            "core/transfer": (100, 99, 101),
        }
        current = {
            "url_build_test_url/file_2": (75, 74, 76),
            "core/transfer": (100, 99, 101),
        }
        self.assertEqual(benchmarks.regressions(baseline, current, 0.1, ("url_",)), [])

    def test_ignored_prefix_does_not_hide_other_regressions(self):
        baseline = {
            "url_build_upload_url": (50, 49, 51),
            "core/transfer": (100, 99, 101),
        }
        current = {
            "url_build_upload_url": (75, 74, 76),
            "core/transfer": (200, 199, 201),
        }
        failures = benchmarks.regressions(baseline, current, 0.1, ("url_",))
        self.assertEqual([name for name, _ in failures], ["core/transfer"])
        self.assertAlmostEqual(failures[0][1], 1.0)

    def test_all_ignored_benchmarks_fail_closed(self):
        with self.assertRaisesRegex(ValueError, "No benchmarks remain"):
            benchmarks.regressions(
                {"url_build_upload_url": (50, 49, 51)},
                {"url_build_upload_url": (75, 74, 76)},
                0.1,
                ("url_",),
            )

    def test_cli_reads_criterion_and_returns_nonzero(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for folder, value in [("baseline", 100), ("current", 200)]:
                path = root / folder / "group" / "benchmark" / "new" / "estimates.json"
                path.parent.mkdir(parents=True)
                path.write_text(
                    json.dumps(
                        {
                            "mean": {
                                "point_estimate": value,
                                "confidence_interval": {
                                    "lower_bound": value - 1,
                                    "upper_bound": value + 1,
                                },
                            }
                        }
                    )
                )
            result = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPT),
                    str(root / "baseline"),
                    str(root / "current"),
                ],
                capture_output=True,
                text=True,
            )
            self.assertEqual(result.returncode, 1)
            self.assertIn("REGRESSION group/benchmark", result.stdout)

    def test_cli_ignores_matching_prefix(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            values = {
                "baseline": {"url_build_upload_url": 50, "core/transfer": 100},
                "current": {"url_build_upload_url": 75, "core/transfer": 100},
            }
            for folder, benchmarks_by_name in values.items():
                for name, value in benchmarks_by_name.items():
                    path = root / folder / name / "new" / "estimates.json"
                    path.parent.mkdir(parents=True)
                    path.write_text(
                        json.dumps(
                            {
                                "mean": {
                                    "point_estimate": value,
                                    "confidence_interval": {
                                        "lower_bound": value - 1,
                                        "upper_bound": value + 1,
                                    },
                                }
                            }
                        )
                    )
            result = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPT),
                    str(root / "baseline"),
                    str(root / "current"),
                    "--ignore-prefix",
                    "url_",
                ],
                capture_output=True,
                text=True,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn(
                "Compared 1 benchmarks; ignored 1; 0 regressions", result.stdout
            )


if __name__ == "__main__":
    unittest.main()
