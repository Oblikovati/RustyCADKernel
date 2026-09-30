#!/usr/bin/env python3
"""Compare all SplitSurface candidates, retaining native crashes as explicit divergences."""

from __future__ import annotations
import json
from pathlib import Path
from porting import snapshot
from toolkit_numeric import NumericOracle, build_probes
from verify_tkmath_numeric import selected_paths
from verify_tkshhealing_numeric import ORACLE as TOOLKIT
from tkshhealing_corpus_split_curve3d import NATIVE_CRASH_REVISION
from tkshhealing_corpus_split_surface import split_surface_partition
from native_crash_accounting import replay_crash, verify_defined


def surface_oracle() -> NumericOracle:
    """Configure isolated surface accounting evidence; surface_oracle()."""
    return NumericOracle(
        "TKShHealing-SplitSurface",
        TOOLKIT.probe_source,
        TOOLKIT.libraries,
        TOOLKIT.rust_package,
        lambda: "",
    )


def verify_split_surface() -> None:
    """Replay exact requests and each native crash without claiming complete parity."""
    source, build = selected_paths()
    revision = snapshot(source)["revision"]
    if revision != NATIVE_CRASH_REVISION:
        raise ValueError(
            f"OCCT revision {revision}; expected {NATIVE_CRASH_REVISION}, reclassify crashes"
        )
    oracle = surface_oracle()
    output = oracle.evidence
    output.mkdir(parents=True, exist_ok=True)
    (output / "report.json").unlink(missing_ok=True)
    native = build_probes(oracle, build, output)
    exact, crashes = split_surface_partition()
    verify_defined(native, oracle.rust_probe, exact, output)
    divergent = [
        replay_crash(native, oracle.rust_probe, request) for request in crashes
    ]
    write_surface_report(output, revision, exact, divergent)


def write_surface_report(
    output: Path, revision: str, exact: list[str], divergent: list[dict[str, str | int]]
) -> None:
    """Keep exact passes and unresolved divergences separate; write_surface_report(out,rev,rows,crashes)."""
    report = {
        "revision": revision,
        "candidates": len(exact) + len(divergent),
        "exact_passed": len(exact),
        "native_crash_divergences": divergent,
        "complete_parity": False,
    }
    (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(
        f"SplitSurface: {len(exact)} exact passed + {len(divergent)} native-crash/Rust-error divergences; complete parity false"
    )


if __name__ == "__main__":
    verify_split_surface()
