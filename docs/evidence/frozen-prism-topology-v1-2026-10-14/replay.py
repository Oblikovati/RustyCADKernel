#!/usr/bin/env python3
"""Replay the private exact topology gate; no native driver or external geometry oracle."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[3]
TOOLCHAIN = Path('/home/vmiguel/.rustup/toolchains/1.99.0-x86_64-unknown-linux-gnu/bin')
LEGACY_SHA256 = '88e28942e1fc6ad301a90f5f07624962d842f0262bde1377103cd3470a871fb4'


def write_json(path: Path, value: object) -> None:
    rendered = json.dumps(value, sort_keys=True, indent=2)
    path.write_text(rendered + '\n')


def source_snapshot() -> dict[str, object]:
    listed = subprocess.check_output(['git', 'ls-files', '-z'], cwd=ROOT).split(b'\0')
    paths = [Path(os.fsdecode(item)) for item in listed if item]
    included = [path for path in paths if not str(path).startswith('docs/evidence/')]
    digests = {str(path): hashlib.sha256((ROOT / path).read_bytes()).hexdigest() for path in included}
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    return {'revision': revision, 'sha256': digests}


def run_gate(output: Path, name: str, command: list[str], env: dict[str, str]) -> dict[str, object]:
    started = time.monotonic()
    with (output / (name + '.log')).open('wb') as stream:
        result = subprocess.run(command, cwd=ROOT, env=env, stdout=stream, stderr=subprocess.STDOUT)
    receipt = {'name': name, 'command': command, 'returncode': result.returncode,
               'elapsed_seconds': time.monotonic() - started}
    write_json(output / (name + '.json'), receipt)
    if result.returncode:
        raise RuntimeError(f'{name} exit {result.returncode}; expected zero, see retained log')
    return receipt


def replay_record(output: Path, name: str, prefix: str) -> bytes:
    lines = (output / (name + '.log')).read_text().splitlines()
    records = [line[len(prefix):] for line in lines if line.startswith(prefix)]
    if len(records) != 1:
        raise RuntimeError(f'{name} has {len(records)} replay records; expected exactly one')
    return bytes.fromhex(records[0])


def gates() -> list[tuple[str, list[str]]]:
    base = ['cargo', 'test', '-p', 'rustycad-tkbo']
    replay = base + ['--lib', 'exact_topology_report_replays_across_threads_and_processes', '--', '--nocapture']
    return [
        ('all-targets', base + ['--all-targets', '--all-features', '--', '--test-threads=1']),
        ('doctests', base + ['--doc', '--all-features']),
        ('docs', ['cargo', 'doc', '-p', 'rustycad-tkbo', '--no-deps', '--all-features']),
        ('clippy', ['cargo', 'clippy', '-p', 'rustycad-tkbo', '--all-targets', '--all-features', '--no-deps', '--', '-D', 'warnings']),
        ('fmt', ['cargo', 'fmt', '--package', 'rustycad-tkbo', '--', '--check']),
        ('process-a', replay), ('process-b', replay),
        ('legacy', base + ['--lib', 'frozen_prism_owned_bytes_replay_across_threads_and_repeated_operations', '--', '--nocapture']),
    ]


def check_replays(output: Path) -> dict[str, object]:
    first = replay_record(output, 'process-a', 'frozen-topology-v1:')
    second = replay_record(output, 'process-b', 'frozen-topology-v1:')
    legacy = replay_record(output, 'legacy', 'frozen-prism-v1:')
    if first != second or hashlib.sha256(legacy).hexdigest() != LEGACY_SHA256:
        raise RuntimeError('replay mismatch; expected exact process equality and unchanged V1 bytes')
    return {'topology_length': len(first), 'topology_sha256': hashlib.sha256(first).hexdigest(),
            'legacy_length': len(legacy), 'legacy_sha256': LEGACY_SHA256}


def execute(output: Path, env: dict[str, str]) -> dict[str, object]:
    receipts = [run_gate(output, name, command, env) for name, command in gates()]
    replay = check_replays(output)
    return {'commands': receipts, 'replay': replay, 'performance_acceptance': False}


def main() -> None:
    output = ROOT / (sys.argv[1] if len(sys.argv) > 1 else 'target/prism-topology-v1/final')
    output.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, PATH=str(TOOLCHAIN) + ':' + os.environ['PATH'], CARGO_BUILD_JOBS='2')
    env['RUSTDOCFLAGS'] = '-D warnings'
    before = source_snapshot()
    write_json(output / 'source-before.json', before)
    try:
        summary = execute(output, env)
    finally:
        after = source_snapshot()
        write_json(output / 'source-after.json', after)
    if before != after:
        raise RuntimeError('source bookend changed; expected identical revision and file hashes')
    write_json(output / 'summary.json', dict(summary, source_unchanged=True, revision=before['revision']))


if __name__ == '__main__':
    main()
