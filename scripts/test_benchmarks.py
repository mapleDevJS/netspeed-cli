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
        self.assertTrue(benchmarks.regressions({"a": (100, 99, 101)}, {"a": (125, 124, 126)}, .1))

    def test_noise_does_not_fail(self):
        self.assertFalse(benchmarks.regressions({"a": (100, 80, 120)}, {"a": (125, 100, 150)}, .1))

    def test_missing_benchmark_fails(self):
        with self.assertRaises(ValueError):
            benchmarks.regressions({"a": (100, 99, 101)}, {}, .1)

    def test_cli_reads_criterion_and_returns_nonzero(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for folder, value in [("baseline", 100), ("current", 130)]:
                path = root / folder / "group" / "benchmark" / "new" / "estimates.json"
                path.parent.mkdir(parents=True)
                path.write_text(json.dumps({"mean": {"point_estimate": value,
                    "confidence_interval": {"lower_bound": value - 1, "upper_bound": value + 1}}}))
            result = subprocess.run([sys.executable, str(SCRIPT), str(root / "baseline"),
                                     str(root / "current")], capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            self.assertIn("REGRESSION group/benchmark", result.stdout)


if __name__ == "__main__":
    unittest.main()
