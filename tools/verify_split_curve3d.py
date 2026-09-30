#!/usr/bin/env python3
"""Account all SplitCurve3d candidates: exact comparisons plus native crash divergences.

This does not classify the crashing paths as parity passes. Each crash is replayed in
an isolated native subprocess, and Rust must retain its explicit construction failure.
"""

from __future__ import annotations
import json
from pathlib import Path
from porting import snapshot
from toolkit_numeric import NumericOracle, build_probes
from verify_tkmath_numeric import selected_paths
from native_crash_accounting import replay_crash, validate_crash, verify_defined
from verify_tkshhealing_numeric import ORACLE as TOOLKIT
from tkshhealing_corpus_split_curve3d import split3d_partition, NATIVE_CRASH_REVISION


def verify_split_curve3d() -> None:
    """Build probes and account all requests without claiming crashing paths pass parity."""
    source, build = selected_paths()
    revision = snapshot(source)["revision"]
    if revision != NATIVE_CRASH_REVISION:
        raise ValueError(
            f"OCCT revision {revision}; expected {NATIVE_CRASH_REVISION}, reclassify crashes"
        )
    oracle = split3d_oracle()
    output = oracle.evidence
    output.mkdir(parents=True, exist_ok=True)
    (output / "report.json").unlink(missing_ok=True)
    native = build_probes(oracle, build, output)
    exact, crashes = split3d_partition()
    verify_defined(native, oracle.rust_probe, exact, output)
    divergent = [
        replay_crash(native, oracle.rust_probe, request) for request in crashes
    ]
    write_split_report(output, revision, exact, divergent)


def split3d_oracle() -> NumericOracle:
    """Configure the isolated accounting probes; split3d_oracle()."""
    return NumericOracle(
        "TKShHealing-SplitCurve3d",
        TOOLKIT.probe_source,
        TOOLKIT.libraries,
        TOOLKIT.rust_package,
        lambda: "",
    )


def write_split_report(
    output: Path, revision: str, exact: list[str], divergent: list[dict[str, str | int]]
) -> None:
    """Record exact passes and unresolved divergences separately; write_split_report(out,revision,exact,divergent)."""
    report = {
        "revision": revision,
        "candidates": len(exact) + len(divergent),
        "exact_passed": len(exact),
        "native_crash_divergences": divergent,
        "complete_parity": False,
    }
    (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(
        f"SplitCurve3d: {len(exact)} exact passed + {len(divergent)} native-crash/Rust-error divergences; complete parity false"
    )


if __name__ == "__main__":
    verify_split_curve3d()
