"""Seal reviewable evidence and changed crate paths after all frozen checks finish."""
import gzip, hashlib, json, pathlib, subprocess
root = pathlib.Path(__file__).resolve().parents[3]
out = pathlib.Path(__file__).resolve().parent
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
gate = json.loads((out / 'final-gate.json').read_text())
assert len(gate['commands']) == 5
assert all(row['exit'] == 0 and row['source_unchanged'] for row in gate['commands'])
for phase in ['final-exact-first', 'final-exact-second']:
    receipt = json.loads((out / (phase + '.json')).read_text())
    assert receipt['source_bookends_equal']
    assert len(json.loads((out / (phase + '-comparison.json')).read_text())) == 8
manifest = json.loads(gzip.decompress((out / 'final-source.json.gz').read_bytes()))
assert all(hashlib.sha256((root / p).read_bytes()).hexdigest() == digest for p, digest in manifest.items())
changed = subprocess.check_output(['git', 'diff', '--name-only', '3563103f1', head, '--', 'crates'], cwd=root, text=True).splitlines()
crate_files = {p: hashlib.sha256((root / p).read_bytes()).hexdigest() for p in changed if (root / p).is_file()}
artifacts = {str(p.relative_to(out)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(out.rglob('*')) if p.is_file() and p.name != 'seal.json'}
result = {'source': gate['source'], 'evidence_head_before_seal': head, 'integration_base': '3563103f1', 'source_files': len(manifest), 'candidate_crate_files': crate_files, 'artifacts': artifacts, 'thread_sanitizer': 'unfulfilled: ordinary-host instrumented controls failed before main; no suppression or waiver', 'scope': 'Private actual annular source/copy/chart/history binding only; no completed solid or publication authority'}
(out / 'seal.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({'source': gate['source'], 'source_files': len(manifest), 'candidate_crate_files': len(crate_files), 'artifacts': len(artifacts)}))
