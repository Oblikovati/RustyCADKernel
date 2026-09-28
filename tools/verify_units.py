#!/usr/bin/env python3
"""Compare unit token spelling, dimensions and conversion bits with the pinned kernel.

Usage: python3 tools/verify_units.py (after building the foundation oracle).
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
    """Every stored unit symbol plus prefix, composite, shifted and regression expressions."""
    dictionary = ROOT/'crates/tkernel/src/units/Units.dat'
    expressions: list[bytes] = []
    lines = dictionary.read_bytes().splitlines()
    position = 0
    while position < len(lines):
        line = lines[position]
        position += 1
        if line.startswith(b'.'):
            position += 3
            continue
        symbol = line[51:78].rstrip()
        if symbol and symbol not in expressions: expressions.append(symbol)
    expressions += [b'mm',b'cm',b'km',b'kg',b'kg/cm/s',b'ton/(m*s**2)',b'tonne/(mm*s**2)',
                    b'sq.km',b'sq.m',b'cu.cm',b'm**2',b'm**3',b'm/s**2',b'kg*(m/s**2)',
                    b'mn',b'min',b'unknown_unit',b'Po',b'km/h',b'mm'+bytes([178]),
                    b'1/m',b'm**(-2)',b'm**2**3',b'(m)',b'2',b'0',b'Pi',b'm m']
    return ''.join(f"{expression.hex()} {value} {target.hex()}\n"
                   for expression in expressions for value in [0.0,1.0,-3.25,1234.5]
                   for target in [expression,b'mm',b'deg.C',b'deg.F',b'kg'])


def compare(oracle: str, rust: str, requests: str) -> None:
    """Require exact result count and bits; compare(native, translated, requests)."""
    if len(oracle.splitlines()) != len(requests.splitlines()):
        raise RuntimeError('Incomplete native unit result')
    if oracle == rust:
        return
    for request, expected, actual in zip(requests.splitlines(), oracle.splitlines(), rust.splitlines()):
        if expected != actual:
            raise RuntimeError(f'Units mismatch {request!r}: OCCT {expected}; Rust {actual}')
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
    output = ROOT/'target/oracle-tkernel/units'
    output.mkdir(parents=True, exist_ok=True)
    (output/'report.json').unlink(missing_ok=True)
    build = ROOT/'target/occt-foundation'
    library = next(build.rglob('libTKernel.so'))
    executable = output/'native-probe'
    run(['c++', '-std=c++17', '-O2', '-I'+str(build/'include/opencascade'),
         str(ROOT/'tools/oracle/units_probe.cxx'), str(library), '-Wl,-rpath,'+str(library.parent),
         '-o', str(executable)], output/'compile.log')
    run(['cargo', 'build', '--locked', '-p', 'rustycad-tkernel', '--example', 'units_probe'], output/'rust-build.log')
    verify_probes(output, executable, inventory['revision'])


def verify_probes(output: Path, executable: Path, revision: str) -> None:
    """Execute both probes on identical persisted input; verify_probes(out, executable, sha)."""
    requests = corpus()
    (output/'inputs.txt').write_text(requests)
    oracle = run([str(executable)], output/'native.txt', requests)
    rust = run([str(ROOT/'target/debug/examples/units_probe')], output/'rust.txt', requests)
    compare(oracle, rust, requests)
    report = {'revision': revision, 'cases_passed': len(requests.splitlines()),
              'input_sha256': hashlib.sha256(requests.encode()).hexdigest(),
              'output_sha256': hashlib.sha256(oracle.encode()).hexdigest()}
    (output/'report.json').write_text(json.dumps(report, indent=2)+'\n')
    print(f"Units: {report['cases_passed']} exact token/dimension/conversion comparisons passed")


if __name__ == '__main__':
    main()
