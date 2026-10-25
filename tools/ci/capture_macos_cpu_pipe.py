#!/usr/bin/env python3
"""Capture independent Darwin probes and the unchanged four-test product host."""
from pathlib import Path
import hashlib
import json
import os
import platform
import subprocess
import sys

ROOT = Path.cwd()
OUT = ROOT / 'target' / 'macos-cpu-pipe'
PRODUCT_SHA = 'd01a59c4e3771601af081e53773fd93beda4c8f0'
PRODUCT = Path(os.environ['RUNNER_TEMP']) / 'martensite-cpu-pipe-product'
CARGO_ENV = {key: value for key, value in os.environ.items() if 'TOKEN' not in key}
CARGO_ENV['CARGO_TARGET_DIR'] = str(OUT / 'cargo')
CARGO_ENV.pop('MARTENSITE_RELEASE_HISTORY_SHA256', None)


def command(arguments: list[str], label: str, cwd: Path = ROOT) -> int:
    """Retain complete stdout/stderr and actual status; command(['clang', '--version'], 'clang')."""
    result = subprocess.run(arguments, cwd=cwd, env=CARGO_ENV, capture_output=True, check=False)
    (OUT / (label + '.stdout.log')).write_bytes(result.stdout)
    (OUT / (label + '.stderr.log')).write_bytes(result.stderr)
    records.append(dict(argv=arguments, cwd=str(cwd), returncode=result.returncode, label=label))
    return result.returncode


OUT.mkdir(parents=True, exist_ok=True)
records: list[dict[str, object]] = []
assert platform.system() == 'Darwin', 'This controlled probe requires native macOS'
assert command(['clang', '-std=c11', '-Wall', '-Wextra', '-Werror', '-pthread',
                str(ROOT / 'tools/ci/probe_macos_cpu_pipe.c'), '-o', str(OUT / 'native-probe')], 'compile') == 0
assert command([str(OUT / 'native-probe')], 'native-probe') == 0
PRODUCT.mkdir()
assert command(['git', 'init', str(PRODUCT)], 'init') == 0
assert command(['git', 'remote', 'add', 'origin', 'https://github.com/Oblikovati/Martensite.git'], 'remote', PRODUCT) == 0
assert command(['git', 'fetch', '--depth=1', 'origin', PRODUCT_SHA], 'fetch', PRODUCT) == 0
assert command(['git', 'checkout', '--detach', PRODUCT_SHA], 'checkout', PRODUCT) == 0
status = command(['cargo', '+1.99.0', 'test', '--locked', '-p', 'martensite-tkernel', '--test',
                  'osd_cpu_limit_signal', '--', '--test-threads=1', '--show-output'], 'original-host', PRODUCT)
receipt = dict(product_sha=PRODUCT_SHA, platform=platform.platform(), machine=platform.machine(),
               records=records, original_host_status=status,
               source_sha256=hashlib.sha256((ROOT / 'tools/ci/probe_macos_cpu_pipe.c').read_bytes()).hexdigest(),
               scope='Diagnostic only; original host failure is retained as failure, never release acceptance.')
(OUT / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
sys.exit(status)
