"""Run the frozen binding gate; outputs never modify crate inputs."""
import gzip, hashlib, json, os, pathlib, re, subprocess, time
root = pathlib.Path(__file__).resolve().parents[3]
out = root / 'docs/evidence/annular-prism-binding-v2-2026-10-14'
env = os.environ.copy()
env['PATH'] = '/home/vmiguel/.cargo/bin:' + env['PATH']
env['CARGO_BUILD_JOBS'] = '2'
paths = subprocess.check_output(['git', 'ls-files', '--cached', '--others', '--exclude-standard', 'crates', 'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml'], cwd=root, text=True).splitlines()
def snapshot():
    return {p: hashlib.sha256((root / p).read_bytes()).hexdigest() for p in sorted(set(paths)) if (root / p).is_file()}
before = snapshot()
(out / 'final-source.json.gz').write_bytes(gzip.compress(json.dumps(before, sort_keys=True).encode(), mtime=0))
commands = [
    ('all-targets', ['test', '-p', 'rustycad-tkbo', '--all-targets', '--all-features', '--', '--test-threads=2']),
    ('docs', ['test', '-p', 'rustycad-tkbo', '--doc', '--all-features']),
    ('clippy', ['clippy', '-p', 'rustycad-tkbo', '--all-targets', '--all-features', '--', '-D', 'warnings']),
    ('rustdoc', ['doc', '-p', 'rustycad-tkbo', '--no-deps', '--all-features']),
    ('fmt', ['fmt', '--all', '--check']),
]
records = []
for label, args in commands:
    assert snapshot() == before, 'input changed before ' + label
    command = ['cargo', '+1.99.0'] + args
    started = time.monotonic()
    run = subprocess.run(command, cwd=root, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    stable = snapshot() == before
    log = 'final-' + label + '.log.gz'
    (out / log).write_bytes(gzip.compress(run.stdout, mtime=0))
    records.append({'label': label, 'command': command, 'exit': run.returncode, 'seconds': time.monotonic() - started, 'source_unchanged': stable, 'log': log, 'results': re.findall(r'test result:.*', run.stdout.decode(errors='replace'))})
    receipt = {'source': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip(), 'source_files': len(before), 'platform': subprocess.check_output(['uname', '-a'], text=True).strip(), 'toolchain': subprocess.check_output(['rustc', '+1.99.0', '-Vv'], env=env, text=True), 'commands': records}
    (out / 'final-gate.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(records[-1]), flush=True)
    assert run.returncode == 0 and stable, 'gate failed: ' + label
