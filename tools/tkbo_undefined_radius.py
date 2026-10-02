"""Mandatory accounting for GetRadius's native uninitialized gce_MakeCirc status."""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
from porting import snapshot
from toolkit_numeric import NumericOracle
from tkbo_algorithms_corpus import algorithm_partition
from verify_tkmath_numeric import run

REVISION='3d097a0328e71b826377d4814ab05ec3c3d23871'
def validate_radius_observation(request: str,native: str,rust: str) -> dict[str,str|bool]:
    """Require the captured pinned result without treating undefined status as portable."""
    expected=native.split();actual=rust.split()
    if len(expected)<9 or len(expected)!=len(actual):
        raise ValueError(f'{request}: native {native!r}, Rust {rust!r}; expected equal complete snapshots')
    if expected!=actual:
        raise ValueError(f'{request}: native {native!r}, Rust {rust!r}; expected pinned result match')
    return dict(request=request,native=native,rust=rust,observed_difference=False,
                classification='native uninitialized gce_MakeCirc status; pinned match, not portable parity')


def verify_undefined_radius(oracle: NumericOracle,source: Path) -> None:
    """Replay all affected inputs with both probe binaries and preserve every observation."""
    revision=snapshot(source)['revision']
    if revision!=REVISION:
        raise ValueError(f'OCCT {revision}; expected {REVISION} for radius status accounting')
    candidates=algorithm_partition()[1]
    output=oracle.evidence.parent/'undefined-radius';output.mkdir(parents=True,exist_ok=True)
    (output/'report.json').unlink(missing_ok=True)
    requests='\n'.join(candidates)+'\n';(output/'inputs.txt').write_text(requests)
    native=run([str(oracle.evidence/'native-probe')],output/'native.txt',requests).splitlines()
    rust=run([str(oracle.rust_probe)],output/'rust.txt',requests).splitlines()
    if len(native)!=len(candidates) or len(rust)!=len(candidates):
        raise ValueError(f'{len(native)} native and {len(rust)} Rust outputs; expected {len(candidates)}')
    records=[validate_radius_observation(*row) for row in zip(candidates,native,rust)]
    write_radius_report(output,source,records)


def write_radius_report(output: Path,source: Path,records: list[dict[str,str|bool]]) -> None:
    """Record undefined inputs and source provenance independently of observed equality."""
    original=source/'src/ModelingData/TKGeomBase/gce/gce_MakeCirc.cxx'
    differences=sum(bool(row['observed_difference']) for row in records)
    pinned_matches=len(records)-differences
    report=dict(revision=REVISION,source=str(original),source_sha256=hashlib.sha256(original.read_bytes()).hexdigest(),
                candidates=len(records),exact_passed=0,pinned_matches=pinned_matches,observed_differences=differences,
                undefined_status_observations=records,complete_parity=False)
    (output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    print(f'TKBO radius status: {len(records)} native undefined-status cases, {pinned_matches} pinned matches, {differences} differences; complete parity false')
