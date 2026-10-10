"""Distribution-based frame timing and device-bound regression budgets."""

from __future__ import annotations
import json
import math
import statistics
from collections.abc import Sequence


def summarize_samples(samples: Sequence[float]) -> dict[str, float]:
    """Summarize complete finite timings; e.g. summarize_samples([1., 2., 3.])."""
    if not samples or any(not math.isfinite(value) or value <= 0 for value in samples):
        raise ValueError(
            f"received timings {samples}; expected positive finite milliseconds"
        )
    ordered = sorted(samples)
    return {
        "median_ms": statistics.median(ordered),
        "mean_ms": statistics.mean(ordered),
        "p95_ms": ordered[math.ceil(0.95 * len(ordered)) - 1],
        "p99_ms": ordered[math.ceil(0.99 * len(ordered)) - 1],
        "minimum_ms": ordered[0],
        "maximum_ms": ordered[-1],
    }


def performance_rows(output: str) -> list[dict[str, object]]:
    """Parse benchmark records and retain raw timings; e.g. performance_rows(stdout)."""
    rows = [
        json.loads(line.removeprefix("MARTENSITE_PERFORMANCE="))
        for line in output.splitlines()
        if line.startswith("MARTENSITE_PERFORMANCE=")
    ]
    expected = {
        "single-triangle",
        "many-draws",
        "resize",
        "ais-display-erase",
        "ais-remove-display",
        "surface-bounds",
        "persistence-groups",
        "line-bounds",
        "curve-presentations",
        "ais-line-appearance",
        "point-sprites",
        "text-glyphs",
        "ais-axis-appearance",
        "plane-trihedron-updates",
        "circle-datum-updates",
        "clipping-plane-updates",
        "homogeneous-position-updates",
        "capping-plane-updates",
        "lit-capping-plane-updates",
        "lighting-updates",
        "window-present",
        "window-resize-present",
    }
    if len(rows) != len(expected) or {row["workload"] for row in rows} != expected:
        raise ValueError(f"received workloads {rows}; expected {expected}")
    for row in rows:
        row.update(summarize_samples(row["samples_ms"]), samples=len(row["samples_ms"]))
    return rows


def regression_failures(
    current: dict[str, object],
    baseline: dict[str, object],
    median_ratio: float,
    p95_ratio: float,
) -> list[str]:
    """Compare only matching device/driver workloads; e.g. regression_failures(now, before, 1.25, 1.5)."""
    if (
        current["device"] != baseline["device"]
        or current["driver_info"] != baseline["driver_info"]
    ):
        return [
            "received different device/driver identity; expected the baseline's exact profile"
        ]
    old = {row["workload"]: row for row in baseline["performance"]}
    failures = []
    for row in current["performance"]:
        previous = old.get(row["workload"])
        if not same_workload(row, previous):
            failures.append(
                f"received unmatched workload {row['workload']}; expected matching baseline configuration"
            )
            continue
        failures.extend(metric_failures(row, previous, median_ratio, p95_ratio))
    return failures


def metric_failures(
    current: dict[str, object],
    baseline: dict[str, object],
    median_ratio: float,
    p95_ratio: float,
) -> list[str]:
    """Apply explicit percentile limits; e.g. metric_failures(now, old, 1.25, 1.5)."""
    values = [
        row[key] for row in (current, baseline) for key in ("median_ms", "p95_ms")
    ]
    if any(not math.isfinite(value) or value <= 0 for value in values):
        raise ValueError(
            f"received percentile timings {values}; expected positive finite values"
        )
    return [
        f"{current['workload']} {metric}={current[metric]:.3f}ms exceeds {baseline[metric] * ratio:.3f}ms"
        for metric, ratio in (("median_ms", median_ratio), ("p95_ms", p95_ratio))
        if current[metric] > baseline[metric] * ratio
    ]


def same_workload(
    current: dict[str, object], previous: dict[str, object] | None
) -> bool:
    """Require equivalent timing configuration; e.g. same_workload(now, baseline)."""
    return (
        previous is not None
        and current.get("frames_per_sample", 1) == previous.get("frames_per_sample", 1)
        and all(
            current[key] == previous[key]
            for key in ("draws", "warmup", "validation", "samples")
        )
    )


def startup_milliseconds(output: str) -> float:
    """Report device/pipeline creation separately from steady frames; e.g. startup_milliseconds(stdout)."""
    samples = [
        float(line.split("=", 1)[1])
        for line in output.splitlines()
        if line.startswith("MARTENSITE_STARTUP_MS=")
    ]
    if len(samples) != 1 or not math.isfinite(samples[0]) or samples[0] <= 0:
        raise ValueError(
            f"received startup samples {samples}; expected one positive finite duration"
        )
    return samples[0]
