#!/usr/bin/env python3
"""Account all SplitCurve3d candidates: exact comparisons plus native crash divergences.

This does not classify the crashing paths as parity passes. Each crash is replayed in
an isolated native subprocess, and Rust must retain its explicit construction failure.
"""

from __future__ import annotations
import hashlib
import json
import subprocess
from pathlib import Path
from porting import snapshot
from toolkit_numeric import NumericOracle, build_probes
from verify_tkmath_numeric import selected_paths, run_probe, compare
from verify_tkshhealing_numeric import ORACLE as TOOLKIT
from tkshhealing_corpus_split_curve3d import split3d_partition, NATIVE_CRASH_REVISION


def replay_crash(native: Path, rust: Path, request: str) -> dict[str, str | int]:
    """Require native SIGSEGV and the documented Rust failure; replay_crash(native,rust,request)."""
    original = subprocess.run(
        [str(native)], input=request + "\n", text=True, capture_output=True, timeout=30
    )
    translated = subprocess.run(
        [str(rust)], input=request + "\n", text=True, capture_output=True, timeout=30
    )
    validate_crash(original.returncode, translated.returncode, translated.stdout)
    return {
        "input": request,
        "input_sha256": hashlib.sha256(request.encode()).hexdigest(),
        "native_returncode": original.returncode,
        "rust_returncode": translated.returncode,
        "rust_exception": "Standard_ConstructionError",
        "rust_output": translated.stdout,
        "classification": "native_crash_rust_typed_failure_divergence",
    }


def validate_crash(native_code: int, rust_code: int, rust_output: str) -> None:
    """Reject changed native behavior or a Rust crash; validate_crash(-11,0,'Standard_ConstructionError')."""
    if native_code != -11:
        raise ValueError(
            f"Native return code {native_code}; expected -11 SIGSEGV, reclassify the divergence"
        )
    if rust_code != 0 or "Standard_ConstructionError" not in rust_output.split():
        raise ValueError(
            f"Rust return code {rust_code}, output {rust_output!r}; expected 0 with Standard_ConstructionError"
        )


def verify_defined(native: Path, rust: Path, requests: list[str], output: Path) -> None:
    """Compare the defined paths exactly; verify_defined(native,rust,requests,output)."""
    inputs = "\n".join(requests) + "\n"
    (output / "inputs.txt").write_text(inputs)
    original = run_probe([str(native)], output / "native.txt", inputs)
    translated = run_probe([str(rust)], output / "rust.txt", inputs)
    compare(original, translated, inputs)


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
