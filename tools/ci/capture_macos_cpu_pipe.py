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
CARGO_ENV['CARGO_TARGET_DIR'] = str(Path(os.environ['RUNNER_TEMP']) / 'martensite-cpu-pipe-build')
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
assert command(['clang', '-std=c11', '-Wall', '-Wextra', '-Werror', '-pthread',
                str(ROOT / 'tools/ci/probe_macos_cpu_helper.c'),
                str(ROOT / 'tools/ci/probe_macos_cpu_helper_child.c'), '-lproc', '-o', str(OUT / 'native-helper')], 'helper-compile') == 0
assert command(['clang', '-std=c11', '-Wall', '-Wextra', '-Werror', '-pthread',
                str(ROOT / 'tools/ci/probe_macos_cpu_reaper.c'),
                str(ROOT / 'tools/ci/probe_macos_cpu_helper_child.c'), '-lproc', '-o', str(OUT / 'native-reaper')], 'reaper-compile') == 0
reaper_status = command([str(OUT / 'native-reaper')], 'native-reaper')
(OUT / 'reaper-receipt.json').write_text(json.dumps(dict(status=reaper_status,
    sources_sha256={name: hashlib.sha256((ROOT / 'tools/ci' / name).read_bytes()).hexdigest()
                   for name in ['probe_macos_cpu_reaper.c', 'probe_macos_cpu_helper_child.c', 'probe_macos_cpu_helper.h']},
    scope='Non-destructive ownership refutation only; no post-reap signal or claim of actual PID reuse.'), indent=2) + '\n')
assert reaper_status == 0
helper_status = command([str(OUT / 'native-helper')], 'native-helper')
helper_sources = {name: hashlib.sha256((ROOT / 'tools/ci' / name).read_bytes()).hexdigest()
                  for name in ['probe_macos_cpu_helper.c', 'probe_macos_cpu_helper_child.c',
                               'probe_macos_cpu_helper.h', 'macos_cpu_helper_source_audit.json']}
(OUT / 'helper-receipt.json').write_text(json.dumps(dict(status=helper_status,
    sources_sha256=helper_sources, platform=platform.platform(), machine=platform.machine(),
    scope='Isolated native prototype with explicit SIGCHLD=SIG_DFL. Numeric-PID cleanup is not production-approved for SIG_IGN, SA_NOCLDWAIT or application reapers.'), indent=2) + '\n')
assert helper_status == 0
assert command(['clang', '-std=c11', '-Wall', '-Wextra', '-Werror', '-pthread',
                str(ROOT / 'tools/ci/probe_macos_cpu_expiry.c'),
                str(ROOT / 'tools/ci/probe_macos_cpu_helper_child.c'), '-lproc', '-o', str(OUT / 'native-expiry')], 'expiry-compile') == 0
expiry_status = command([str(OUT / 'native-expiry')], 'native-expiry')
(OUT / 'expiry-receipt.json').write_text(json.dumps(dict(status=expiry_status,
    sources_sha256={name: hashlib.sha256((ROOT / 'tools/ci' / name).read_bytes()).hexdigest()
                   for name in ['probe_macos_cpu_expiry.c', 'probe_macos_cpu_helper_child.c',
                                'probe_macos_cpu_helper.h', 'macos_cpu_expiry_source_audit.json']},
    scope='Experimental Mac15 mechanism only. Public deprecated vfork semantics require Mac12+; product minimum unresolved. Parent expiry has no helper-PID action; platform housekeeping is trusted, not mathematically bounded.'), indent=2) + '\n')
assert expiry_status == 0
PRODUCT.mkdir()
assert command(['git', 'init', str(PRODUCT)], 'init') == 0
assert command(['git', 'remote', 'add', 'origin', 'https://github.com/Oblikovati/Martensite.git'], 'remote', PRODUCT) == 0
assert command(['git', 'fetch', '--depth=1', 'origin', PRODUCT_SHA], 'fetch', PRODUCT) == 0
assert command(['git', 'checkout', '--detach', PRODUCT_SHA], 'checkout', PRODUCT) == 0
status = command(['cargo', '+1.99.0', 'test', '--locked', '-p', 'martensite-tkernel', '--test',
                  'osd_cpu_limit_signal', '--', '--test-threads=1', '--show-output'], 'original-host', PRODUCT)
receipt = dict(product_sha=PRODUCT_SHA, platform=platform.platform(), machine=platform.machine(),
               records=records, original_host_status=status, native_helper_status=helper_status,
               helper_sources_sha256=helper_sources,
               source_sha256=hashlib.sha256((ROOT / 'tools/ci/probe_macos_cpu_pipe.c').read_bytes()).hexdigest(),
               scope='Diagnostic only; original host failure is retained as failure, never release acceptance.')
(OUT / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
sys.exit(status)
