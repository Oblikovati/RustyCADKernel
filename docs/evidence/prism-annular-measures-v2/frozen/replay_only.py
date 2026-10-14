#!/usr/bin/env python3
"""Repair only payload-line parsing after the completed frozen matrix; never repeat its suites."""
import gzip
import hashlib
import json
import platform
import subprocess
import run as gate


def original_matrix():
    original = json.loads(gzip.decompress((gate.OUT / 'source-before.json.gz').read_bytes()))
    after = json.loads(gzip.decompress((gate.OUT / 'matrix-source-after.json.gz').read_bytes()))
    assert original == after
    current = gate.hashes()
    changed = [p for p in original if current[p] != original[p]]
    assert changed == ['docs/evidence/prism-annular-measures-v2/frozen/run.py'], changed
    added = sorted(set(current) - set(original))
    assert added == ['docs/evidence/prism-annular-measures-v2/frozen/replay_only.py'], added
    steps = json.loads((gate.OUT / 'matrix-progress.json').read_text())
    assert [step['label'] for step in steps[:5]] == ['tests', 'doctests', 'clippy', 'format', 'docs']
    assert all(step['exit'] == 0 for step in steps)
    return original, current, steps


def receipt(original, before, steps, replays):
    return {'matrix_revision': '50a287e78', 'replay_runner_revision': gate.git('rev-parse', 'HEAD').decode().strip(),
            'platform': platform.platform(), 'jobs': 2,
            'rustc': subprocess.check_output(['/home/vmiguel/.cargo/bin/rustc', '+1.99.0', '-vV']).decode(),
            'matrix_steps': steps, 'replay_steps': gate.STEPS, 'replays': replays,
            'matrix_bookends_equal': True, 'matrix_tracked_inputs': len(original),
            'replay_bookends_equal': True, 'replay_tracked_inputs': len(before),
            'crate_inputs_unchanged_between_runs': True,
            'harness_repair': 'Accept exact payload prefix after the Rust single-thread test-name prefix; retained original parser refusal.',
            'scope': 'TKMath additive policy validator and TKBO annular conditional measures',
            'limitations': ['Private scalar evidence; no physical volume, body validity, public operation or release claim.',
                           'TSan acceptance remains incomplete: recorded host startup failure; this suite was not run under TSan.',
                           'No new OCCT oracle, storage redesign or previous-release workload benchmark.']}


def main():
    original, before, steps = original_matrix()
    gate.save('replay-source-before.json', json.dumps(before, sort_keys=True).encode())
    replays = gate.replays()
    after = gate.hashes()
    gate.save('replay-source-after.json', json.dumps(after, sort_keys=True).encode())
    assert before == after
    report = receipt(original, before, steps, replays)
    (gate.OUT / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Replay-only supplement complete; original full matrix preserved', flush=True)


if __name__ == '__main__':
    main()
