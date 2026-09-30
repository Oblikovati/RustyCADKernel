#!/usr/bin/env python3
"""Record circle-inspector unchecked slots separately from defined exact parity."""
from __future__ import annotations
import json
import resource
from pathlib import Path
from native_crash_accounting import replay_crash
from porting import snapshot
from toolkit_numeric import NumericOracle
from verify_tkmath_numeric import selected_paths
from verify_tkmesh_numeric import ORACLE as MESH_ORACLE
from verify_tkmesh_vertex_undefined import REVISION, empty_defined_corpus, replay_uninitialized, vertex_probes


def candidates() -> list[str]:
    """Retain both mutable lookup and inspection at negative/uninitialized indices."""
    return [f'mesh_circle_invalid {operation} {index}' for operation in range(2) for index in (-1,1)]


def verify_circle_undefined(use_existing: bool = False) -> None:
    """Require native index gaps and explicit Rust bounds failures; verify_circle_undefined()."""
    source,build=selected_paths()
    revision=snapshot(source)['revision']
    if revision!=REVISION:
        raise ValueError(f'OCCT {revision}; expected {REVISION} for circle index accounting')
    resource.setrlimit(resource.RLIMIT_CORE,(0,0))
    oracle=NumericOracle('TKMesh-circle-undefined',MESH_ORACLE.probe_source,
        MESH_ORACLE.libraries,MESH_ORACLE.rust_package,empty_defined_corpus)
    oracle.evidence.mkdir(parents=True,exist_ok=True)
    (oracle.evidence/'report.json').unlink(missing_ok=True)
    native,rust=vertex_probes(oracle,build,use_existing)
    records=[replay_crash(native,rust,row,'Standard_OutOfRange') if row.endswith(' -1')
        else replay_uninitialized(native,rust,row) for row in candidates()]
    write_report(oracle.evidence,records)


def write_report(output: Path,records: list[dict[str,str|int]]) -> None:
    """Keep undefined slots outside the exact-parity count; write_report(path,records)."""
    (output/'inputs.txt').write_text('\n'.join(candidates())+'\n')
    (output/'report.json').write_text(json.dumps(dict(revision=REVISION,candidates=len(records),
        exact_passed=0,undefined_index_divergences=records,complete_parity=False),indent=2)+'\n')
    print(f'TKMesh circle bounds: {len(records)} undefined-index divergences; complete parity false')

if __name__=='__main__':
    verify_circle_undefined()
