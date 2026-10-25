#!/usr/bin/env python3
"""Retain actual native Rust CPU controls on the two supported Mac architectures."""
from pathlib import Path
import hashlib
import json
import os
import platform
import subprocess
import sys

ROOT = Path.cwd()
OUT = ROOT / 'target' / 'macos-cpu-pipe'
PRODUCT_SHA = '3425741478dededeab63feb00bb51ffd7511f68c'
PRODUCT = Path(os.environ['RUNNER_TEMP']) / 'martensite-cpu-rust-product'
CARGO_ENV = {key: value for key, value in os.environ.items() if 'TOKEN' not in key}
CARGO_ENV.pop('MARTENSITE_RELEASE_HISTORY_SHA256', None)
CARGO_ENV['CARGO_TARGET_DIR'] = str(Path(os.environ['RUNNER_TEMP']) / 'martensite-cpu-rust-build')
CARGO_ENV['CARGO_BUILD_JOBS'] = '4'
OUT.mkdir(parents=True, exist_ok=True)
records: list[dict[str, object]] = []


def command(arguments: list[str], label: str, cwd: Path = ROOT) -> int:
    """Bind raw output and actual exit status; command(['rustc', '--version'], 'compiler')."""
    result = subprocess.run(arguments, cwd=cwd, env=CARGO_ENV, capture_output=True, check=False)
    (OUT / (label + '.stdout.log')).write_bytes(result.stdout)
    (OUT / (label + '.stderr.log')).write_bytes(result.stderr)
    records.append(dict(argv=arguments, cwd=str(cwd), returncode=result.returncode, label=label))
    return result.returncode


assert platform.system() == 'Darwin', 'This native product test requires macOS'
PRODUCT.mkdir()
assert command(['git', 'init', str(PRODUCT)], 'rust-init') == 0
assert command(['git', 'remote', 'add', 'origin', 'https://github.com/Oblikovati/Martensite.git'], 'rust-remote', PRODUCT) == 0
assert command(['git', 'fetch', '--depth=1', 'origin', PRODUCT_SHA], 'rust-fetch', PRODUCT) == 0
assert command(['git', 'checkout', '--detach', PRODUCT_SHA], 'rust-checkout', PRODUCT) == 0
compiler_status = command(['rustc', '+1.99.0', '-vV'], 'rust-compiler', PRODUCT)
unit_status = command(['cargo', '+1.99.0', 'test', '--locked', '-p', 'martensite-tkernel',
                       '--lib', 'osd::timing::signal_native::cpu_budget', '--',
                       '--test-threads=1', '--show-output'], 'rust-budget-unit', PRODUCT)
host_status = command(['cargo', '+1.99.0', 'test', '--locked', '-p', 'martensite-tkernel',
                       '--test', 'osd_cpu_limit_signal', '--', '--test-threads=1',
                       '--show-output'], 'rust-original-host', PRODUCT)
unit_output = (OUT / 'rust-budget-unit.stdout.log').read_text()
host_output = (OUT / 'rust-original-host.stdout.log').read_text()
unit_coverage = 'test result: ok. 5 passed; 0 failed;' in unit_output
host_coverage = 'test result: ok. 4 passed; 0 failed;' in host_output
sources = PRODUCT / 'crates' / 'martensite-tkernel' / 'src' / 'osd' / 'timing'
source_hashes = {str(path.relative_to(PRODUCT)): hashlib.sha256(path.read_bytes()).hexdigest()
                 for path in sorted(sources.rglob('*.rs'))}
receipt = dict(product_sha=PRODUCT_SHA, platform=platform.platform(), machine=platform.machine(),
               compiler_status=compiler_status, unit_status=unit_status, original_host_status=host_status,
               unit_exact_five_passed=unit_coverage, original_exact_four_passed=host_coverage,
               records=records, source_sha256=source_hashes,
               scope='Native Rust focused controls only. Oldest macOS support policy and the exact final release matrix remain separate gates.')
(OUT / 'rust-receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
sys.exit(0 if compiler_status == unit_status == host_status == 0 and unit_coverage and host_coverage else 1)
