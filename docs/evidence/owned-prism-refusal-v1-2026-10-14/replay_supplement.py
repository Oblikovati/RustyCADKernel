#!/usr/bin/env python3
"""Replay fixed refused/accepted outcomes against the completed source-frozen gate binary."""
import argparse, ast, gzip, hashlib, json, os, pathlib, re, subprocess
parser = argparse.ArgumentParser()
parser.add_argument('--label', required=True)
args = parser.parse_args()
root = pathlib.Path.cwd()
out = root / 'target/prism-ordered-validation-v1' / args.label
baseline = root / 'target/prism-ordered-validation-v1/owned-refusal-prefix'
prior = root / 'docs/evidence/prism-publication-v1-2026-10-14'
freeze = json.loads((out / 'source-before.json').read_text())
match = re.search(r'Running unittests src/lib.rs \(([^)]+)\)', (out / 'all-targets.log').read_text())
binary = root / match.group(1)

def digest(path):
    content = path.read_bytes()
    result = hashlib.sha256(content).hexdigest()
    return result

def bookend():
    tracked = subprocess.check_output(['git', 'ls-files', '-z']).decode().split('\0')
    files = {f: digest(pathlib.Path(f)) for f in sorted(tracked) if f and pathlib.Path(f).is_file() and (pathlib.Path(f).suffix in ['.rs', '.toml', '.lock'] or f.startswith('.cargo/') or 'rust-toolchain' in f)}
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    return {'revision': revision, 'source_files': files, 'binary_sha256': digest(binary)}

def run_pair(name, test, marker):
    contents = []
    for repeat in [1, 2]:
        cmd = [str(binary), '--exact', 'sketch_region_prism::' + test, '--nocapture']
        run = subprocess.run(cmd, capture_output=True)
        (out / f'{name}-supplement-{repeat}.log').write_bytes(run.stdout + run.stderr)
        rows = [line[len(marker):] for line in run.stdout.decode().splitlines() if line.startswith(marker)]
        assert run.returncode == 0 and len(rows) == 1, (name, repeat, run.returncode)
        content = bytes(ast.literal_eval(rows[0])) if name == 'semantic' else bytes.fromhex(rows[0])
        (out / f'{name}-supplement-{repeat}.bin').write_bytes(content)
        records.append({'name': name, 'repeat': repeat, 'command': cmd, 'returncode': run.returncode, 'bytes': len(content), 'sha256': hashlib.sha256(content).hexdigest()})
        contents.append(content)
    assert contents[0] and contents[0] == contents[1], name
    return contents[0]

before = bookend()
assert before['source_files'] == freeze['source_files'] and before['revision'] == freeze['revision']
records = []
tool = '/home/vmiguel/.rustup/toolchains/1.99.0-x86_64-unknown-linux-gnu/bin'
env = os.environ.copy()
env.update(PATH=tool + ':' + env['PATH'], CARGO_BUILD_JOBS='2', RUSTDOCFLAGS='-D warnings')
doc_cmd = [tool + '/cargo', 'doc', '-p', 'rustycad-tkbo', '--no-deps', '--all-features']
with (out / 'docs.log').open('w') as log:
    docs = subprocess.run(doc_cmd, env=env, stdout=log, stderr=subprocess.STDOUT)
assert docs.returncode == 0, docs.returncode
cases = [
 ('owned-refusal', 'frozen::owned_refusal::tests::case_tests::owned_refusal_process_receipt', 'owned-prism-refusal-v1:'),
 ('publication', 'publication::replay_tests::publication_process_replay', 'named-prism-publication-v1='),
 ('history', 'frozen::complete_history::operation_tests::complete_prism_history_replay_across_processes_and_threads', 'complete-prism-history-v1:'),
 ('semantic', 'owned_request::names::tests::semantic_replay_probe', 'SEMANTIC_REPLAY='),
 ('measures', 'frozen::joined_measures::tests::replay_tests::joined_measure_process_replay', 'joined-measures-v1='),
 ('lineage', 'frozen::lineage::replay_tests::prism_copy_lineage_owned_bytes_replay_across_processes_and_threads', 'prism-copy-lineage-v1:'),
]
comparisons = {}
for name, test, marker in cases:
    content = run_pair(name, test, marker)
    if name == 'owned-refusal':
        expected = (baseline / 'owned-refusal-process-1.bin').read_bytes()
    elif name == 'semantic':
        expected = (root / 'crates/tkbo/src/sketch_region_prism/owned_request/names/fixtures/names-v1.bin').read_bytes()
    else:
        expected = gzip.decompress((prior / f'{name}-1.bin.gz').read_bytes())
    comparisons[name] = content == expected
for name in ['ordered', 'containment-v1', 'topology', 'join', 'orientation', 'tolerance']:
    comparisons[name] = (out / f'{name}-process-1.bin').read_bytes() == (baseline / f'{name}-process-1.bin').read_bytes()
after = bookend()
result = {'before': before, 'after': after, 'docs': {'command': doc_cmd, 'returncode': docs.returncode, 'rustdocflags': '-D warnings'}, 'records': records, 'equal_reviewed_receipts': comparisons}
(out / 'supplement.json').write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
assert before == after and all(comparisons.values()), comparisons
print(json.dumps({'runs': len(records), 'equal_reviewed_receipts': comparisons}))
