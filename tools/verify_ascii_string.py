#!/usr/bin/env python3
"""Compare string hash, formatting, trimming and prefix conversion with OCCT."""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
import random
import struct
from porting import ROOT, snapshot
from verify_strtod import run, source_path


def corpus() -> str:
    """Return reproducible requests covering IEEE values and arbitrary byte hashes."""
    rng = random.Random(11758)
    bits = [0, 1, 0x8000000000000000, 0x7ff0000000000000, 0xfff0000000000000,
            0x7ff8000000000000, 0xfff8000000000000, 0x7fefffffffffffff]
    bits += [struct.unpack('>Q', struct.pack('>d', value))[0] for value in
             [3.3, 1e-4, 1e-5, 1e6, 999999.5, 0.9999995, 0.00009999995]]
    requests = [f'f {value:016x}' for value in bits + [rng.getrandbits(64) for _ in range(2048)]]
    hashes = [b'', b'A', bytes([255]), bytes([128,255,127]), b'Test', b'a\0b']
    hashes += [rng.randbytes(rng.randrange(0,256)) for _ in range(512)]
    requests += ['h '+(value.hex() or '-') for value in hashes]
    requests += ['s '+(value.hex() or '-') for value in conversion_cases()]
    requests += ['r '+value.hex() for value in [b'a\0b', b'3.3\0junk']]
    return '\n'.join(requests)+'\n'


def conversion_cases() -> list[bytes]:
    """Include embedded terminators and overflow, which ordinary legacy examples omit."""
    values = ['', ' ', '+', '-', '42x', '123.4', '123e4', '123x.', '3.3!', '  -42',
              '2147483648', '9223372036854775807', '9223372036854775808',
              '-9223372036854775808', '-9223372036854775809', '9'*100,
              'inf', '-nan', '0x1p2', 'Hello世界', 'Ä\t\n', 'Привет   ', 'a\0b', '3.3\0junk', '']
    return [value.encode() for value in values]


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
    output = ROOT/'target/oracle-tkernel/ascii-string'
    output.mkdir(parents=True, exist_ok=True)
    (output/'report.json').unlink(missing_ok=True)
    build = ROOT/'target/occt-foundation'
    library = next(build.rglob('libTKernel.so'))
    executable = output/'native-probe'
    run(['c++', '-std=c++17', '-O2', '-I'+str(build/'include/opencascade'),
         str(ROOT/'tools/oracle/ascii_string_probe.cxx'), str(library), '-Wl,-rpath,'+str(library.parent),
         '-o', str(executable)], output/'compile.log')
    run(['cargo', 'build', '--locked', '-p', 'rustycad-tkernel', '--example', 'ascii_string_probe'], output/'rust-build.log')
    verify(output, executable, inventory['revision'])


def verify(output: Path, executable: Path, revision: str) -> None:
    """Persist the input, both outputs and their hashes; verify(out, executable, sha)."""
    requests = corpus()
    (output/'inputs.txt').write_text(requests)
    native = run([str(executable)], output/'native.txt', requests)
    rust = run([str(ROOT/'target/debug/examples/ascii_string_probe')], output/'rust.txt', requests)
    compare(native, rust, requests)
    report = {'revision': revision, 'cases_passed': len(requests.splitlines()),
              'input_sha256': hashlib.sha256(requests.encode()).hexdigest(),
              'output_sha256': hashlib.sha256(native.encode()).hexdigest()}
    (output/'report.json').write_text(json.dumps(report, indent=2)+'\n')
    print(f"AsciiString: {report['cases_passed']} exact-output comparisons passed")


if __name__ == '__main__':
    main()
