#!/usr/bin/env python3
"""Frozen candidate checks: full affected toolkits and exact independent-process receipts."""
import ast
import gzip
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import time

ROOT = Path.cwd()
OUT = Path(__file__).resolve().parent
ENV = {**os.environ, 'CARGO_BUILD_JOBS': '2', 'PATH': '/home/vmiguel/.cargo/bin:' + os.environ['PATH']}
CARGO = ['/home/vmiguel/.cargo/bin/cargo', '+1.99.0']
STEPS = []


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def hashes():
    paths = git('ls-files', '-z').decode().split('\0')
    return {p: hashlib.sha256((ROOT / p).read_bytes()).hexdigest() for p in paths if p}


def save(name, payload):
    (OUT / (name + '.gz')).write_bytes(gzip.compress(payload, mtime=0))


def run(label, args, extra=None):
    print('start ' + label, flush=True)
    start = time.monotonic()
    log = OUT / (label + '.live.log')
    with log.open('wb') as stream:
        process = subprocess.run(args, env={**ENV, **(extra or {})}, stdout=stream, stderr=subprocess.STDOUT)
    output = log.read_bytes()
    save(label + '.log', output)
    log.unlink()
    counts = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', output.decode(errors='replace'))
    step = {'label': label, 'command': args, 'exit': process.returncode, 'seconds': time.monotonic() - start,
            'test_counts': [list(map(int, row)) for row in counts]}
    STEPS.append(step)
    (OUT / 'progress.json').write_text(json.dumps(STEPS, indent=2) + '\n')
    print(json.dumps(step), flush=True)
    assert process.returncode == 0, label
    return output.decode()


def process_pair(label, selector, prefix, kind, baseline):
    values = []
    for index in [1, 2]:
        output = run(f'{label}-{index}', CARGO + ['test', '-p', 'rustycad-tkbo', '--lib', '--all-features', '--locked',
                     selector, '--', '--exact', '--nocapture', '--test-threads=1'])
        assert '1 passed; 0 failed' in output
        rows = [line.removeprefix(prefix) for line in output.splitlines() if line.startswith(prefix)]
        assert len(rows) == 1, (selector, len(rows))
        payload = bytes.fromhex(rows[0]) if kind == 'hex' else bytes(ast.literal_eval(rows[0]))
        values.append(payload)
        save(f'{label}-{index}.bin', payload)
    assert values[0] == values[1], label
    if baseline:
        assert values[0] == gzip.decompress((ROOT / baseline).read_bytes()), label
    return {'label': label, 'bytes': len(values[0]), 'sha256': hashlib.sha256(values[0]).hexdigest(),
            'processes_equal': True, 'baseline': baseline, 'baseline_unchanged': bool(baseline)}


def checks():
    packages = ['-p', 'rustycad-tkmath', '-p', 'rustycad-tkbo', '--all-features', '--locked']
    run('tests', CARGO + ['test', '--all-targets'] + packages)
    run('doctests', CARGO + ['test', '--doc'] + packages)
    run('clippy', CARGO + ['clippy', '--all-targets'] + packages + ['--', '-D', 'warnings'])
    run('format', CARGO + ['fmt', '--all', '--', '--check'])
    run('docs', CARGO + ['doc', '--no-deps'] + packages, {'RUSTDOCFLAGS': '-D warnings'})


def replays():
    base = 'sketch_region_prism::frozen::'
    evidence = 'docs/evidence/prism-annular-measures-v2/'
    return [process_pair('annular-measures', base + 'annular_measures::tests::baseline::annular_measure_process_receipt',
                         'annular-measures-v2=', 'hex', evidence + 'initial/baseline.bin.gz'),
            process_pair('joined-measures-v1', base + 'joined_measures::tests::replay_tests::joined_measure_process_replay',
                         'joined-measures-v1=', 'hex', evidence + 'codec/before.bin.gz'),
            process_pair('annular-domain', base + 'annular_join::replay_tests::annular_frozen_join_owned_process_receipt',
                         'named-annular-frozen-domain-v2=', 'hex', None),
            process_pair('annular-orientation', base + 'annular_orientation::lifecycle_tests::annular_orientation_replay_probe',
                         'ANNULAR_ORIENTATION_REPLAY=', 'list', None)]


def main():
    assert not git('status', '--porcelain').strip(), 'start from committed clean source'
    revision = git('rev-parse', 'HEAD').decode().strip()
    before = hashes()
    save('source-before.json', json.dumps(before, sort_keys=True).encode())
    checks()
    receipts = replays()
    after = hashes()
    save('source-after.json', json.dumps(after, sort_keys=True).encode())
    assert before == after, 'tracked inputs changed during frozen checks'
    receipt = {'revision': revision, 'platform': platform.platform(), 'jobs': 2,
               'rustc': subprocess.check_output(['/home/vmiguel/.cargo/bin/rustc', '+1.99.0', '-vV']).decode(),
               'steps': STEPS, 'replays': receipts, 'tracked_inputs': len(before), 'bookends_equal': True,
               'scope': 'TKMath additive policy validator and TKBO annular conditional measures',
               'limitations': ['Private scalar evidence only; no physical volume, body validity, public operation or release claim.',
                               'TSan acceptance remains incomplete: recorded host startup failure; this suite was not run under TSan.',
                               'No new OCCT oracle, performance redesign or previous-release workload benchmark.']}
    (OUT / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print('frozen gate complete ' + revision, flush=True)


if __name__ == '__main__':
    main()
