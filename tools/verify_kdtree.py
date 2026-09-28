#!/usr/bin/env python3
"""Compare exact KDTree indices, traversal ordering and binary64 results with OCCT.

Usage: python3 tools/verify_kdtree.py (after building the foundation oracle).
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
    """Fixed clouds, tie-heavy permutations, leaf boundaries and seeded random queries."""
    rng = random.Random(23497)
    lines: list[str] = []
    for dimension in [2, 3]:
        for size in [0, 1, 2, 3, 4, 7, 31, 32, 33, 63, 64, 65, 100, 257, 1024]:
            for distribution in range(8):
                points: list[list[float]] = []
                for index in range(size):
                    if distribution == 0: coordinates = [0.0] * dimension
                    elif distribution == 1: coordinates = [float(index)] * dimension
                    elif distribution == 2: coordinates = [float(size-index)] * dimension
                    elif distribution == 3: coordinates = [float(min(index,size-index))] * dimension
                    elif distribution == 4: coordinates = [float(rng.randrange(-2,3)) for _ in range(dimension)]
                    else: coordinates = [rng.uniform(-100,100) for _ in range(dimension)]
                    points.append(coordinates+[rng.choice([0.0, 1.0, 20.0, rng.uniform(-5,50)])])
                for query_number in range(4):
                    query = [0.0]*dimension if query_number == 0 else [rng.uniform(-120,120) for _ in range(dimension)]
                    minimum = [coordinate-25 for coordinate in query]
                    maximum = [coordinate+25 for coordinate in query]
                    radius = [-1.0, 0.0, 30.0, 10000.0][query_number]
                    tolerance = [-0.1, 0.0, 0.001, 0.1][query_number]
                    k = [0, 1, 7, size+3][query_number]
                    fields = [dimension,size]+[value for point in points for value in point]+query+minimum+maximum+[radius,tolerance,k]
                    lines.append(' '.join(map(str,fields)))
    for dimension in [2,3]:
        for size in [33,65]:
            points = [[index*1e200]+[0.0]*(dimension-1)+[0.0] for index in range(size)]
            query = [-1e200]*dimension
            fields = [dimension,size]+[value for point in points for value in point]+query+[-1e201]*dimension+[1e201]*dimension+[1e201,0.0,size]
            lines.append(' '.join(map(str,fields)))
    return '\n'.join(lines)+'\n'


def compare(oracle: str, rust: str, requests: str) -> None:
    """Require exact result count and bits; compare(native, translated, requests)."""
    if len(oracle.splitlines()) != len(requests.splitlines()):
        raise RuntimeError('Incomplete native KDTree result')
    if oracle == rust:
        return
    for request, expected, actual in zip(requests.splitlines(), oracle.splitlines(), rust.splitlines()):
        if expected != actual:
            raise RuntimeError(f'KDTree mismatch {request!r}: OCCT {expected}; Rust {actual}')
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
    output = ROOT/'target/oracle-tkernel/kdtree'
    output.mkdir(parents=True, exist_ok=True)
    (output/'report.json').unlink(missing_ok=True)
    build = ROOT/'target/occt-foundation'
    library = next(build.rglob('libTKernel.so'))
    executable = output/'native-probe'
    run(['c++', '-std=c++17', '-O2', '-I'+str(build/'include/opencascade'),
         str(ROOT/'tools/oracle/kdtree_probe.cxx'), str(library), '-Wl,-rpath,'+str(library.parent),
         '-o', str(executable)], output/'compile.log')
    run(['cargo', 'build', '--locked', '-p', 'rustycad-tkernel', '--example', 'kdtree_probe'], output/'rust-build.log')
    verify_probes(output, executable, inventory['revision'])


def verify_probes(output: Path, executable: Path, revision: str) -> None:
    """Execute both probes on identical persisted input; verify_probes(out, executable, sha)."""
    requests = corpus()
    (output/'inputs.txt').write_text(requests)
    oracle = run([str(executable)], output/'native.txt', requests)
    rust = run([str(ROOT/'target/debug/examples/kdtree_probe')], output/'rust.txt', requests)
    compare(oracle, rust, requests)
    report = {'revision': revision, 'cases_passed': len(requests.splitlines()),
              'input_sha256': hashlib.sha256(requests.encode()).hexdigest(),
              'output_sha256': hashlib.sha256(oracle.encode()).hexdigest()}
    (output/'report.json').write_text(json.dumps(report, indent=2)+'\n')
    print(f"KDTree: {report['cases_passed']} exact index/order/binary64 comparisons passed")


if __name__ == '__main__':
    main()
