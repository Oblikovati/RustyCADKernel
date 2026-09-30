#!/usr/bin/env python3
"""Retain native unchecked element reads and off-by-one writes as explicit divergences."""
from __future__ import annotations
import json
import resource
from pathlib import Path
from native_crash_accounting import replay_crash
from porting import snapshot
from toolkit_numeric import NumericOracle, build_probes
from verify_tkmath_numeric import selected_paths
from verify_tkmesh_numeric import ORACLE as MESH_ORACLE
from verify_tkmesh_vertex_undefined import REVISION, empty_defined_corpus, replay_uninitialized


def candidates() -> list[str]:
    """Keep invalid node/element reads and both native replacement-write paths."""
    return [f"mesh_delaun_invalid {operation} {index}"
            for operation in (0, 1) for index in (-1, 0, 2)] + [
                "mesh_delaun_invalid 2 1", "mesh_delaun_invalid 3 1"]


def write_report(output: Path, records: list[dict[str, str | int]]) -> None:
    """Publish non-parity evidence independently of exact comparison counts."""
    report = {"revision": REVISION, "candidates": len(records), "exact_passed": 0,
              "undefined_index_divergences": records, "complete_parity": False}
    (output / "inputs.txt").write_text("\n".join(candidates()) + "\n")
    (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(f"TKMesh Delaun bounds: {len(records)} undefined-index divergences; complete parity false")


def replay_candidate(native: Path, rust: Path, row: str) -> dict[str, str | int]:
    """Require the observed native fault for negative/zero reads, keeping all other UB explicit."""
    if row.split()[1] in ("0", "1") and row.split()[2] in ("-1", "0"):
        return replay_crash(native, rust, row, "Standard_OutOfRange")
    return replay_uninitialized(native, rust, row)


def verify_delaun_undefined(use_existing: bool = False) -> None:
    """Replay the immutable native candidate inventory; verify_delaun_undefined()."""
    source, build = selected_paths()
    revision = snapshot(source)["revision"]
    if revision != REVISION:
        raise ValueError(f"OCCT {revision}; expected {REVISION} for unchecked-index accounting")
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    oracle = NumericOracle("TKMesh-delaun-undefined", MESH_ORACLE.probe_source,
                          MESH_ORACLE.libraries, MESH_ORACLE.rust_package, empty_defined_corpus)
    oracle.evidence.mkdir(parents=True, exist_ok=True)
    (oracle.evidence / "report.json").unlink(missing_ok=True)
    native = MESH_ORACLE.evidence / "native-probe" if use_existing else build_probes(oracle, build, oracle.evidence)
    records = [replay_candidate(native, MESH_ORACLE.rust_probe, row) for row in candidates()]
    write_report(oracle.evidence, records)


if __name__ == "__main__":
    verify_delaun_undefined()
