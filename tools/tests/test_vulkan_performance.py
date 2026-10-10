"""Performance regressions fail against an explicitly compatible baseline."""

import json
import unittest
from pathlib import Path
from verify_vulkan import HarnessConfig, check_baseline, parse_arguments, configuration
from vulkan_performance import summarize_samples, performance_rows, regression_failures


class VulkanPerformanceTests(unittest.TestCase):
    def test_percentiles_and_nonfinite_samples(self) -> None:
        result = summarize_samples(list(range(1, 101)))
        self.assertEqual(result["median_ms"], 50.5)
        self.assertEqual(result["p95_ms"], 95.0)
        self.assertEqual(result["p99_ms"], 99.0)
        for samples in ([], [0.0], [-1.0], [float("nan")], [float("inf")]):
            with self.assertRaises(ValueError):
                summarize_samples(samples)

    def test_workload_records_require_all_scenarios(self) -> None:
        lines = [
            "MARTENSITE_PERFORMANCE="
            + json.dumps({"workload": name, "samples_ms": [1.0, 2.0, 3.0]})
            for name in ("single-triangle", "many-draws", "resize")
        ]
        rows = performance_rows("\n".join(lines))
        self.assertEqual(len(rows), 3)
        self.assertEqual(rows[0]["median_ms"], 2.0)
        with self.assertRaises(ValueError):
            performance_rows(lines[0])

    def test_latency_regression_and_wrong_device_are_failures(self) -> None:
        row = {
            "workload": "test",
            "draws": 1,
            "warmup": 10,
            "samples": 40,
            "validation": False,
            "median_ms": 1.0,
            "p95_ms": 1.0,
        }
        baseline = {"device": "GPU", "driver_info": "driver", "performance": [row]}
        same = regression_failures(baseline, baseline, 1.25, 1.5)
        self.assertEqual(same, [])
        slower = {**baseline, "performance": [{**row, "median_ms": 2.0, "p95_ms": 2.0}]}
        self.assertEqual(len(regression_failures(slower, baseline, 1.25, 1.5)), 2)
        invalid = {**baseline, "performance": [{**row, "median_ms": float("nan")}]}
        with self.assertRaises(ValueError):
            regression_failures(invalid, baseline, 1.25, 1.5)
        wrong = {**baseline, "device": "software"}
        self.assertIn(
            "different device", regression_failures(wrong, baseline, 1.25, 1.5)[0]
        )
        changed = {**baseline, "performance": [{**row, "draws": 2}]}
        self.assertIn(
            "unmatched workload", regression_failures(changed, baseline, 1.25, 1.5)[0]
        )

    def test_invalid_budgets_and_missing_baselines_fail(self) -> None:
        for ratio in (float("nan"), float("inf"), 0.9):
            with self.assertRaises(ValueError):
                HarnessConfig(Path("unused"), median_ratio=ratio)
        with self.assertRaises(ValueError):
            HarnessConfig(Path("unused"), samples=0)
        self.assertEqual(check_baseline({}, HarnessConfig(Path("unused"))), [])
        failures = check_baseline(
            {"profile": "host"},
            HarnessConfig(Path("unused"), baseline={"profiles": []}),
        )
        self.assertIn("no successful baseline", failures[0])

    def test_cli_defaults_match_recorded_performance_workloads(self) -> None:
        args = parse_arguments(
            ["--source", "source", "--build", "build", "--output", "out.json"]
        )
        config = configuration(args)
        self.assertEqual((config.samples, config.warmup), (100, 20))
        self.assertEqual(config.profiles, ("lavapipe", "host"))
