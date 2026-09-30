#!/usr/bin/env python3
"""Retain unchecked native vertex-index candidates as divergences, never exact passes."""
from __future__ import annotations
import hashlib
import json
import resource
import subprocess
from pathlib import Path
from native_crash_accounting import replay_crash
from porting import snapshot
from toolkit_numeric import NumericOracle, build_probes
from verify_tkmath_numeric import selected_paths
from verify_tkmesh_numeric import ORACLE as MESH_ORACLE

REVISION = "3d097a0328e71b826377d4814ab05ec3c3d23871"


def candidates() -> list[str]:
    """Keep negative, zero and allocated-but-uninitialized slots; candidates()."""
    return [f"mesh_vertex_invalid {index} {operation}"
            for index in (-1, 0, 2) for operation in range(3)]


def empty_defined_corpus() -> str:
    """This inventory has no exact-parity rows; empty_defined_corpus()."""
    return ""


def replay_uninitialized(native: Path, rust: Path, request: str) -> dict[str, str | int]:
    """Record unchecked allocated-slot access separately; replay_uninitialized(native,rust,row)."""
    original = subprocess.run([str(native)], input=request + "\n", text=True,
                              capture_output=True, timeout=30)
    translated = subprocess.run([str(rust)], input=request + "\n", text=True,
                                capture_output=True, timeout=30)
    if original.returncode not in (0, -11):
        raise ValueError(f"Native code {original.returncode}; expected 0 or -11 for unchecked slot access")
    if translated.returncode != 0 or "Standard_OutOfRange" not in translated.stdout.split():
        raise ValueError(f"Rust code {translated.returncode}, output {translated.stdout!r}; expected 0 and Standard_OutOfRange")
    return {"input": request, "input_sha256": hashlib.sha256(request.encode()).hexdigest(),
            "native_returncode": original.returncode, "native_output": original.stdout,
            "rust_returncode": translated.returncode, "rust_output": translated.stdout,
            "rust_exception": "Standard_OutOfRange",
            "classification": "native_uninitialized_slot_rust_bounds_failure_divergence"}


def verify_vertex_undefined(use_existing: bool = False) -> None:
    """Replay the pinned invalid-index inventory; verify_vertex_undefined()."""
    source, build = selected_paths()
    revision = snapshot(source)["revision"]
    if revision != REVISION:
        raise ValueError(f"OCCT {revision}; expected {REVISION} for unchecked-index accounting")
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    oracle = NumericOracle("TKMesh-vertex-undefined", MESH_ORACLE.probe_source,
                          MESH_ORACLE.libraries, MESH_ORACLE.rust_package, empty_defined_corpus)
    oracle.evidence.mkdir(parents=True, exist_ok=True)
    (oracle.evidence / "report.json").unlink(missing_ok=True)
    native, rust = vertex_probes(oracle, build, use_existing)
    records = replay_candidates(native, rust)
    write_report(oracle.evidence, records)


def vertex_probes(oracle: NumericOracle, build: Path, use_existing: bool) -> tuple[Path, Path]:
    """Reuse the just-verified defined-domain twins, or build standalone twins."""
    if use_existing:
        return MESH_ORACLE.evidence / "native-probe", MESH_ORACLE.rust_probe
    return build_probes(oracle, build, oracle.evidence), oracle.rust_probe


def replay_candidates(native: Path, rust: Path) -> list[dict[str, str | int]]:
    """Recheck all discovered candidates and require native crashes for negative slots."""
    records: list[dict[str, str | int]] = []
    for request in candidates():
        if request.split()[1] in ("-1", "0"):
            records.append(replay_crash(native, rust, request, "Standard_OutOfRange"))
        else:
            records.append(replay_uninitialized(native, rust, request))
    return records


def write_report(output: Path, records: list[dict[str, str | int]]) -> None:
    """Publish explicit non-parity accounting; write_report(output,records)."""
    report = {"revision": REVISION, "candidates": len(records), "exact_passed": 0,
              "undefined_index_divergences": records, "complete_parity": False}
    (output / "inputs.txt").write_text("\n".join(candidates()) + "\n")
    (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(f"TKMesh vertex bounds: {len(records)} undefined-index divergences; complete parity false")


if __name__ == "__main__":
    verify_vertex_undefined()
