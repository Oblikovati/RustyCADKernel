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
RADIUS_SENTINEL='4625478292286210048'


def validate_radius_observation(request: str,native: str,rust: str) -> dict[str,str|bool]:
    """Validate unaffected fields without treating an incidental status match as parity."""
    expected=native.split();actual=rust.split()
    if len(expected)<9 or len(expected)!=len(actual):
        raise ValueError(f'{request}: native {native!r}, Rust {rust!r}; expected equal complete snapshots')
    if actual[1:3]!=['0','0'] or expected[1] not in ('0','2','-1'):
        raise ValueError(f'{request}: native {native!r}, Rust {rust!r}; expected radius statuses 0/2/-1 and Rust zero radius')
    radius='0' if expected[1]=='0' else RADIUS_SENTINEL
    if expected[2]!=radius or expected[:1]+expected[3:]!=actual[:1]+actual[3:]:
        raise ValueError(f'{request}: native {native!r}, Rust {rust!r}; expected only status/radius variation')
    return dict(request=request,native=native,rust=rust,observed_difference=expected!=actual,
                classification='native uninitialized gce_MakeCirc status; never an exact parity pass')


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
    report=dict(revision=REVISION,source=str(original),source_sha256=hashlib.sha256(original.read_bytes()).hexdigest(),
                candidates=len(records),exact_passed=0,observed_differences=differences,
                undefined_status_observations=records,complete_parity=False)
    (output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    print(f'TKBO radius status: {len(records)} native undefined-status cases, {differences} observed differences; complete parity false')
