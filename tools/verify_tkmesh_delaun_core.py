#!/usr/bin/env python3
"""Defined Delaun core parity and separately reproduced native collinear crashes."""
import hashlib
import json
import resource
from native_crash_accounting import verify_crash_oracle,replay_crash
from toolkit_numeric import NumericOracle
from verify_tkmath_numeric import selected_paths
from verify_tkmesh_numeric import ORACLE as MESH_ORACLE
from tkmesh_delaun_core_corpus import corpus,partition

REVISION='3d097a0328e71b826377d4814ab05ec3c3d23871'

def verify_delaun_core() -> None:
    """Run 2208 retained candidates against unchanged native library members."""
    source,_=selected_paths()
    resource.setrlimit(resource.RLIMIT_CORE,(0,0))
    oracle=NumericOracle('TKMesh-delaun-core',MESH_ORACLE.probe_source,
        MESH_ORACLE.libraries,MESH_ORACLE.rust_package,corpus)
    verify_crash_oracle(oracle,partition,REVISION,'Standard_OutOfRange')
    report=oracle.evidence/'report.json'
    evidence=json.loads(report.read_text())
    relative='src/ModelingAlgorithms/TKMesh/BRepMesh/BRepMesh_Delaun.cxx'
    evidence['private_helper_source']=relative
    evidence['private_helper_source_sha256']=hashlib.sha256((source/relative).read_bytes()).hexdigest()
    evidence['private_helper_access']='Explicit-instantiation member pointers; unchanged shared library; -std=c++17 -O2'
    evidence['original_legacy_declarations_claimed']=0
    evidence['null_mesh_divergences']=[replay_crash(oracle.evidence/'native-probe',oracle.rust_probe,
        f'mesh_delaun_null {op}','Standard_NullObject') for op in range(9)]
    report.write_text(json.dumps(evidence,indent=2)+'\n')

if __name__=='__main__':
    verify_delaun_core()
