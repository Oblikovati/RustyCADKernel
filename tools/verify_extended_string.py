#!/usr/bin/env python3
"""Compare every UTF-16 code unit, conversion and hash with OCCT."""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
import random
from porting import ROOT, snapshot
from verify_strtod import run, source_path


def corpus() -> str:
    """Exercise every single UTF-16 code unit and seeded multi-unit sequences."""
    requests = ['-'] + [f'{unit:04x}' for unit in range(65536)]
    requests += ['0041d8000042dc00d83dde00', 'd800dc00', 'dbffdfff', '004100000042']
    rng = random.Random(22744)
    requests += [''.join(f'{rng.randrange(65536):04x}' for _ in range(rng.randrange(1,32)))
                 for _ in range(1024)]
    return '\n'.join(requests)+'\n'


def compare(expected: str, actual: str, requests: str) -> None:
    """Require one exact result per request, without tolerance."""
    if len(expected.splitlines()) != len(requests.splitlines()):
        raise RuntimeError('Native string result count differs from request count')
    if expected == actual:
        return
    for request, native, rust in zip(requests.splitlines(), expected.splitlines(), actual.splitlines()):
        if native != rust:
            raise RuntimeError(f'String mismatch {request!r}: OCCT {native!r}; Rust {rust!r}')
    raise RuntimeError('String probe result counts differ')


def main() -> None:
    """Build probes and write success evidence only after all comparisons agree."""
    inventory = snapshot(source_path())
    output = ROOT/'target/oracle-tkernel/extended-string'
    output.mkdir(parents=True, exist_ok=True)
    (output/'report.json').unlink(missing_ok=True)
    build = ROOT/'target/occt-foundation'
    library = next(build.rglob('libTKernel.so'))
    executable = output/'native-probe'
    run(['c++', '-std=c++17', '-O2', '-I'+str(build/'include/opencascade'),
         str(ROOT/'tools/oracle/extended_string_probe.cxx'), str(library), '-Wl,-rpath,'+str(library.parent),
         '-o', str(executable)], output/'compile.log')
    run(['cargo', 'build', '--locked', '-p', 'rustycad-tkernel', '--example', 'extended_string_probe'], output/'rust-build.log')
    verify(output, executable, inventory['revision'])


def verify(output: Path, executable: Path, revision: str) -> None:
    """Persist the input, both outputs and their hashes; verify(out, executable, sha)."""
    requests = corpus()
    (output/'inputs.txt').write_text(requests)
    native = run([str(executable)], output/'native.txt', requests)
    rust = run([str(ROOT/'target/debug/examples/extended_string_probe')], output/'rust.txt', requests)
    compare(native, rust, requests)
    report = {'revision': revision, 'cases_passed': len(requests.splitlines()),
              'input_sha256': hashlib.sha256(requests.encode()).hexdigest(),
              'output_sha256': hashlib.sha256(native.encode()).hexdigest()}
    (output/'report.json').write_text(json.dumps(report, indent=2)+'\n')
    print(f"ExtendedString: {report['cases_passed']} exact-output comparisons passed")


if __name__ == '__main__':
    main()
