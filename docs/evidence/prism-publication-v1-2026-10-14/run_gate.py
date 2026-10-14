#!/usr/bin/env python3
"""Frozen TKBO-only publication verification; immutable raw logs retain all failures."""
import argparse
import gzip
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import time

PARSER = argparse.ArgumentParser()
PARSER.add_argument('--label', required=True)
ROOT = Path.cwd()
OUT = ROOT / 'target/prism-publication-final' / PARSER.parse_args().label
CARGO = ['/home/vmiguel/.cargo/bin/cargo', '+1.99.0']
ENV = dict(os.environ, CARGO_BUILD_JOBS='2')
ENV['PATH'] = '/home/vmiguel/.cargo/bin:' + ENV['PATH']
OUT.mkdir(parents=True, exist_ok=True)


def save(name, value):
    (OUT / name).write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')


def snapshot(name):
    paths = subprocess.check_output(['git', 'ls-files', '-z']).decode().split('\0')
    hashes = {p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
              for p in sorted(paths) if p and Path(p).is_file()}
    value = {'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
             'status': subprocess.check_output(['git', 'status', '--short'], text=True),
             'all_tracked_input_hashes': hashes}
    save(name, value)
    return value


def execute(name, command):
    start = time.monotonic()
    with (OUT / (name + '.log')).open('w') as log:
        run = subprocess.run(command, env=ENV, stdout=log, stderr=subprocess.STDOUT)
    summaries = re.findall(r'test result: .*', (OUT / (name + '.log')).read_text())
    result = {'name': name, 'command': command, 'returncode': run.returncode,
              'seconds': time.monotonic() - start, 'test_summaries': summaries}
    print(json.dumps(result), flush=True)
    return result


def commands():
    settings = ['-p', 'rustycad-tkbo', '--all-features']
    tasks = [('all-targets', ['test', *settings, '--all-targets', '--no-fail-fast', '--', '--test-threads=1']),
             ('doctests', ['test', *settings, '--doc']),
             ('clippy', ['clippy', *settings, '--all-targets', '--', '-D', 'warnings']),
             ('docs', ['doc', *settings, '--no-deps']),
             ('format', ['fmt', '--all', '--', '--check'])]
    records = []
    for name, arguments in tasks:
        records.append(execute(name, CARGO + arguments))
        save('commands.json', records)
    return records


CASES = [
 ('publication', 'publication::replay_tests::publication_process_replay', 'named-prism-publication-v1='),
 ('containment', 'frozen::replay_tests::frozen_prism_owned_bytes_replay_across_threads_and_repeated_operations', 'frozen-prism-v1:'),
 ('topology', 'frozen::topology::replay_tests::exact_topology_report_replays_across_threads_and_processes', 'frozen-topology-v1:'),
 ('join', 'frozen::proof_join::tests::joined_math_byte_receipt', 'same-body-prism-proof-v1:'),
 ('orientation', 'frozen::joined_orientation::tests::replay_tests::joined_orientation_process_receipt', 'joined-orientation-v1:'),
 ('tolerance', 'exact_tolerance::joined_replay_tests::exact_tolerance_bytes_replay_across_threads_and_operations', 'prism-exact-tolerance-v2:'),
 ('ordered', 'frozen::ordered_validation::replay_tests::ordered_validation_process_receipt', 'ordered-validation-v1:'),
 ('measures', 'frozen::joined_measures::tests::replay_tests::joined_measure_process_replay', 'joined-measures-v1='),
 ('history', 'frozen::complete_history::operation_tests::complete_prism_history_replay_across_processes_and_threads', 'complete-prism-history-v1:'),
 ('lineage', 'frozen::lineage::replay_tests::prism_copy_lineage_owned_bytes_replay_across_processes_and_threads', 'prism-copy-lineage-v1:'),
]


def replay_once(binary, case, repeat):
    name, test, marker = case
    command = [str(binary), '--exact', 'sketch_region_prism::' + test, '--nocapture']
    run = subprocess.run(command, env=ENV, capture_output=True)
    (OUT / f'{name}-{repeat}.log.gz').write_bytes(gzip.compress(run.stdout + run.stderr, mtime=0))
    rows = [line[len(marker):] for line in run.stdout.decode().splitlines() if line.startswith(marker)]
    if run.returncode != 0 or len(rows) != 1:
        raise RuntimeError(f'{name}: exit={run.returncode}, marker rows={len(rows)}; expected successful one-row replay')
    receipt = bytes.fromhex(rows[0])
    (OUT / f'{name}-{repeat}.bin.gz').write_bytes(gzip.compress(receipt, mtime=0))
    return receipt, {'command': command, 'bytes': len(receipt), 'sha256': hashlib.sha256(receipt).hexdigest()}


def check_baseline(name, receipt):
    if name == 'containment':
        expected = (ROOT / 'crates/tkbo/src/sketch_region_prism/frozen/fixtures/containment-v1.bin').read_bytes()
    elif name == 'publication':
        path = ROOT / 'docs/evidence/prism-publication-v1-development-2026-10-14/publication-66513b361.bin.gz'
        expected = gzip.decompress(path.read_bytes())
    else:
        return
    if receipt != expected:
        raise RuntimeError(f'{name}: full bytes differ from reviewed unchanged baseline')


def replays():
    text = (OUT / 'all-targets.log').read_text()
    binary = ROOT / re.search(r'Running unittests src/lib.rs \(([^)]+)\)', text).group(1)
    before = hashlib.sha256(binary.read_bytes()).hexdigest()
    records = []
    for case in CASES:
        first, one = replay_once(binary, case, 1)
        second, two = replay_once(binary, case, 2)
        if first != second:
            raise RuntimeError(f'{case[0]}: two complete process receipts differ')
        check_baseline(case[0], first)
        records.append({'name': case[0], 'first': one, 'second': two, 'exact_equal': True})
        save('replays.json', records)
    assert hashlib.sha256(binary.read_bytes()).hexdigest() == before
    return {'binary': str(binary), 'sha256': before, 'pairs': len(records)}


def main():
    before = snapshot('inputs-before.json')
    checks = commands()
    replay = replays()
    after = snapshot('inputs-after.json')
    unchanged = before == after
    result = {'source_revision': before['revision'], 'all_tracked_inputs_unchanged': unchanged,
              'host': platform.platform(), 'jobs': 2, 'commands': checks, 'replays': replay,
              'toolchain': subprocess.check_output(['/home/vmiguel/.cargo/bin/rustc', '+1.99.0', '-Vv'], text=True),
              'tsan': 'incomplete: publication suite not built or run; retained host startup control fails before program marker',
              'scope': 'TKBO only; TKMath/TKBRep sources unchanged; no public release or application acceptance'}
    save('result.json', result)
    if not unchanged or any(row['returncode'] for row in checks):
        raise SystemExit(1)


if __name__ == '__main__':
    main()
