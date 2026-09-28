#!/usr/bin/env python3
"""Compare exact Strtod result bits and end pointers against the pinned native kernel.

Usage: python3 tools/verify_strtod.py (after building the foundation oracle).
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
    """Exercise prefix grammar, special values, rounding and exponent boundaries."""
    cases = ['', ' ', '+', '-', '.', ' .e2', '1e', '1e+', '1e-', '  +1.25tail', '-0',
             '0x', '0x.', '0x.p1', '0x1', '0x1p', '0x1p+', '0x.8p2', '-0x0p999',
             'inf', '-Infinity', '+INFinitYtail', 'nan', '-nan', 'nan()', 'nan(1)',
             'nan(0x1)', 'nan(abcdef)', 'nan(123 456)', 'nan(1 2 3)', 'nan(foo)', 'nan(1',
             '0x1p-1074', '0x1p-1075', '0x1.0000000000001p-1075',
             '0x1.fffffffffffffp1023', '0x1.fffffffffffff8p1023', '0x1.fffffffffffffp-1023',
             '5e-324', '2.2250738585072014e-308', '1.7976931348623157e308', '1e99999', '1e-99999']
    rng = random.Random(22980)
    for _ in range(1024):
        digits = ''.join(rng.choice('0123456789abcdef') for _ in range(rng.randrange(1,80)))
        split = rng.randrange(len(digits)+1)
        cases.append(f"-0x{digits[:split]}.{digits[split:]}p{rng.randrange(-1200,1200)}tail")
        cases.append(f" {rng.randrange(-10**30,10**30)}.{rng.randrange(10**30)}e{rng.randrange(-350,350)}suffix")
    return '\n'.join(cases)+'\n'


def compare(oracle: str, rust: str, requests: str) -> None:
    """Require exact result count and bits; compare(native, translated, requests)."""
    if len(oracle.splitlines()) != len(requests.splitlines()):
        raise RuntimeError('Incomplete native numeric result')
    if oracle == rust:
        return
    for request, expected, actual in zip(requests.splitlines(), oracle.splitlines(), rust.splitlines()):
        if expected != actual:
            raise RuntimeError(f'Strtod mismatch {request!r}: OCCT {expected}; Rust {actual}')
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
    output = ROOT/'target/oracle-tkernel/strtod'
    output.mkdir(parents=True, exist_ok=True)
    (output/'report.json').unlink(missing_ok=True)
    build = ROOT/'target/occt-foundation'
    library = next(build.rglob('libTKernel.so'))
    executable = output/'native-probe'
    run(['c++', '-std=c++17', '-O2', '-I'+str(build/'include/opencascade'),
         str(ROOT/'tools/oracle/strtod_probe.cxx'), str(library), '-Wl,-rpath,'+str(library.parent),
         '-o', str(executable)], output/'compile.log')
    run(['cargo', 'build', '--locked', '-p', 'rustycad-tkernel', '--example', 'strtod_probe'], output/'rust-build.log')
    verify_probes(output, executable, inventory['revision'])


def verify_probes(output: Path, executable: Path, revision: str) -> None:
    """Execute both probes on identical persisted input; verify_probes(out, executable, sha)."""
    requests = corpus()
    (output/'inputs.txt').write_text(requests)
    oracle = run([str(executable)], output/'native.txt', requests)
    rust = run([str(ROOT/'target/debug/examples/strtod_probe')], output/'rust.txt', requests)
    compare(oracle, rust, requests)
    report = {'revision': revision, 'cases_passed': len(requests.splitlines()),
              'input_sha256': hashlib.sha256(requests.encode()).hexdigest(),
              'output_sha256': hashlib.sha256(oracle.encode()).hexdigest()}
    (output/'report.json').write_text(json.dumps(report, indent=2)+'\n')
    print(f"Strtod: {report['cases_passed']} exact binary64/end-offset comparisons passed")


if __name__ == '__main__':
    main()
