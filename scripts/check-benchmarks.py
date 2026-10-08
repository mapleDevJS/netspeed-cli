#!/usr/bin/env python3
"""Compare Criterion estimates; fail when a slowdown exceeds noise and 10%."""

import argparse
import json
import math
from pathlib import Path


def estimates(root, name):
    result = {}
    for path in Path(root).glob(f"**/{name}/estimates.json"):
        mean = json.loads(path.read_text())["mean"]
        point = mean["point_estimate"]
        low = mean["confidence_interval"]["lower_bound"]
        high = mean["confidence_interval"]["upper_bound"]
        if not all(math.isfinite(v) and v > 0 for v in (point, low, high)):
            raise ValueError(f"Invalid estimate: {path}")
        result[str(path.parent.parent.relative_to(root))] = (point, low, high)
    if not result:
        raise ValueError(f"No Criterion {name} estimates found in {root}")
    return result


def regressions(baseline, current, threshold):
    missing = baseline.keys() - current.keys()
    if missing:
        raise ValueError(f"Missing benchmarks: {', '.join(sorted(missing))}")
    return [
        (name, current[name][0] / old[0] - 1)
        for name, old in baseline.items()
        if current[name][1] > old[2] * (1 + threshold)
    ]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline", type=Path)
    parser.add_argument("current", type=Path)
    parser.add_argument("--baseline-name", default="new")
    parser.add_argument("--threshold", type=float, default=0.10)
    args = parser.parse_args()
    if not math.isfinite(args.threshold) or args.threshold < 0:
        parser.error("threshold must be finite and nonnegative")
    try:
        old = estimates(args.baseline, args.baseline_name)
        new = estimates(args.current, "new")
        failures = regressions(old, new, args.threshold)
    except (ValueError, KeyError, OSError) as error:
        parser.exit(1, f"Benchmark comparison failed: {error}\n")
    for name, change in failures:
        print(f"REGRESSION {name}: {change:+.1%}")
    print(f"Compared {len(old)} benchmarks; {len(failures)} regressions")
    return bool(failures)


if __name__ == "__main__":
    raise SystemExit(main())
