#!/usr/bin/env python3
"""Keep null-mesh selector crashes separate from defined-domain exact parity."""
from __future__ import annotations
import resource
from native_crash_accounting import replay_crash, write_crash_report
from porting import snapshot
from toolkit_numeric import NumericOracle
from verify_tkmath_numeric import selected_paths
from verify_tkmesh_numeric import ORACLE as MESH_ORACLE
from verify_tkmesh_vertex_undefined import REVISION, empty_defined_corpus, vertex_probes


def candidates() -> list[str]:
    """Every query overload dereferences the native null mesh; candidates()."""
    return [f"mesh_selector_invalid {operation}" for operation in range(7)]


def verify_selector_undefined(use_existing: bool = False) -> None:
    """Require all seven native crashes and explicit Rust NullObject failures."""
    source, build = selected_paths()
    revision = snapshot(source)["revision"]
    if revision != REVISION:
        raise ValueError(f"OCCT {revision}; expected {REVISION} for null-selector accounting")
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    oracle = NumericOracle("TKMesh-selector-undefined", MESH_ORACLE.probe_source,
                          MESH_ORACLE.libraries, MESH_ORACLE.rust_package, empty_defined_corpus)
    oracle.evidence.mkdir(parents=True, exist_ok=True)
    (oracle.evidence / "report.json").unlink(missing_ok=True)
    native, rust = vertex_probes(oracle, build, use_existing)
    rows = candidates()
    (oracle.evidence / "inputs.txt").write_text("\n".join(rows) + "\n")
    records = [replay_crash(native, rust, row, "Standard_NullObject") for row in rows]
    write_crash_report(oracle.evidence, oracle.toolkit, revision, [], records)


if __name__ == "__main__":
    verify_selector_undefined()
