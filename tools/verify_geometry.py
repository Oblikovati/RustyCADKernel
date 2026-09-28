#!/usr/bin/env python3
"""Compare dump parsing and translated geometry with the pinned kernel.

Usage: python3 tools/verify_units.py (after building the geometry dependency closure).
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import random
import subprocess
from porting import ROOT, snapshot


def run(command: list[str], log: Path, requests: str | None = None) -> str:
    """Capture a checked probe invocation; run(command, log, input_text)."""
    environment = dict(os.environ, LC_ALL='C', LANG='C')
    environment['PATH'] = str(Path.home()/'.cargo/bin')+os.pathsep+environment['PATH']
    result = subprocess.run(command, input=requests, text=True, capture_output=True,
                            cwd=ROOT, env=environment, check=False)
    log.write_text(result.stdout+result.stderr)
    result.check_returncode()
    return result.stdout


def corpus() -> str:
    """Exercise valid and malformed fragments plus native translation rounding."""
    requests = []
    vectors = ['[1, 2, 3]', '[1.25, -2e3, .5]', '[1, bad, 3]', '[1, 2, bad]',
               '[1, 2, 3', '[1, 2]', '[1,2,3]', '[, 2, 3]', '[]', '',
               '[1junk, 2, 3]', '(1, 2, 3]', '[1, 2, 3, 4]', '[1, 2, 3]tail']
    for text in vectors:
        for pos in [-1, 0, 1, 2, len(text)+1]:
            requests.append(f'vector {pos} {text.encode().hex() or "-"}')
        point = '\"gp_Pnt\": '+text
        requests.append(f'point 1 {point.encode().hex()}')
    for text in ['', '42', '42, other', '42}tail', '}tail', ', next', 'x}t, z',
                 '\"gp_Pnt\": [1, 2, 3]', ', \"gp_Pnt\": [1, 2, 3]', ', ', 'wrong']:
        for op in ['key', 'scalar']:
            for pos in [-1, 0, 1, 2, len(text)+1]:
                requests.append(f'{op} {pos} {text.encode().hex() or "-"}')
    rng = random.Random(25545)
    for _ in range(128):
        triple = ' '.join(str(rng.uniform(-1e5,1e5)) for _ in range(3))
        requests.append(f'dump {triple}')
        requests.append(f'translation {triple} {rng.randint(-1000,1000)}')
    for power in [-1000,-4,-3,-2,-1,0,1,2,3,4,1000]:
        requests.append(f'translation 0.1 -0.2 0.3 {power}')
    return '\n'.join(requests)+'\n'


def compare(oracle: str, rust: str, requests: str) -> None:
    """Require exact result count and bits; compare(native, translated, requests)."""
    if len(oracle.splitlines()) != len(requests.splitlines()):
        raise RuntimeError('Incomplete native geometry result')
    if oracle == rust:
        return
    for request, expected, actual in zip(requests.splitlines(), oracle.splitlines(), rust.splitlines()):
        if expected != actual:
            raise RuntimeError(f'Geometry mismatch {request!r}: OCCT {expected}; Rust {actual}')
    raise RuntimeError('Numeric result counts differ')


def source_path() -> Path:
    """Resolve the explicitly selected pinned OCCT checkout; source_path()."""
    parser = argparse.ArgumentParser()
    parser.add_argument('--source', type=Path, default=ROOT/'../OCCT')
    return parser.parse_args().source.resolve()


def main() -> None:
    """Build probes, compare all cases, and write evidence only on success."""
    source = source_path()
    inventory = snapshot(source)
    output = ROOT/'target/oracle-tkernel/geometry'
    output.mkdir(parents=True, exist_ok=True)
    (output/'report.json').unlink(missing_ok=True)
    build = ROOT/'target/occt-tkernel-full'
    libraries = [next(build.rglob('lib'+name+'.so')) for name in ['TKTopAlgo','TKGeomAlgo','TKBRep','TKGeomBase','TKG3d','TKG2d','TKMath','TKernel']]
    library = libraries[-1]
    executable = output/'native-probe'
    run(['c++', '-std=c++17', '-O2', '-I'+str(build/'include/opencascade'),
         str(ROOT/'tools/oracle/geometry_probe.cxx'), *map(str,libraries), '-Wl,-rpath,'+str(library.parent),
         '-o', str(executable)], output/'compile.log')
    run(['cargo', 'build', '--locked', '-p', 'rustycad-tkernel', '--example', 'geometry_probe'], output/'rust-build.log')
    verify_probes(output, executable, inventory['revision'])


def verify_probes(output: Path, executable: Path, revision: str) -> None:
    """Execute both probes on identical persisted input; verify_probes(out, executable, sha)."""
    requests = corpus()
    (output/'inputs.txt').write_text(requests)
    oracle = run([str(executable)], output/'native.txt', requests)
    rust = run([str(ROOT/'target/debug/examples/geometry_probe')], output/'rust.txt', requests)
    compare(oracle, rust, requests)
    report = {'revision': revision, 'cases_passed': len(requests.splitlines()),
              'input_sha256': hashlib.sha256(requests.encode()).hexdigest(),
              'output_sha256': hashlib.sha256(oracle.encode()).hexdigest()}
    (output/'report.json').write_text(json.dumps(report, indent=2)+'\n')
    print(f"Geometry: {report['cases_passed']} exact parser/geometry comparisons passed")


if __name__ == '__main__':
    main()
