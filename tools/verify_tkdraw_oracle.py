#!/usr/bin/env python3
"""Compare the Rust DRAWEXE with native OCCT DRAWEXE on the DRAW oracle scripts.

Usage:
  python3 tools/verify_tkdraw_oracle.py --native-build <occt build> [--record] [--jobs N]

Every crates/tkdraw/tests/oracle/*.tcl script runs in batch mode (`-b -f script`) on
both executables with an empty DRAWDEFAULT, so that neither reads DrawResources or
loads plugins. Without --record the native output must equal the recorded
`<script>.expected` file and the Rust output must equal the native one. --record
rewrites the `.expected` files from the native run (the Rust integration test
`drawexe_oracle` then checks the Rust executable against them without a native build).
"""
from __future__ import annotations

import argparse
import concurrent.futures
import difflib
import os
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
ORACLE_DIR = ROOT / "crates" / "tkdraw" / "tests" / "oracle"
TIMEOUT_SECONDS = 120
MEMORY_LIMIT_KB = 8_000_000


def native_environment(build: Path) -> dict[str, str]:
    """The environment of the native build's env.sh (library paths, CASROOT)."""
    script = f'source "{build}/env.sh" i >/dev/null 2>&1; env -0'
    output = subprocess.run(["bash", "-c", script], check=True, capture_output=True).stdout
    pairs = (item.split(b"=", 1) for item in output.split(b"\0") if b"=" in item)
    return {key.decode(): value.decode(errors="replace") for key, value in pairs}


def native_executable(build: Path) -> Path:
    """The DRAWEXE of the native build."""
    for candidate in (build / "lin64" / "gcc" / "bini" / "DRAWEXE", build / "bin" / "DRAWEXE"):
        if candidate.exists():
            return candidate
    raise SystemExit(f"no DRAWEXE under {build}; expected lin64/gcc/bini/DRAWEXE or bin/DRAWEXE")


def rust_executable() -> Path:
    """Builds and returns the Rust drawexe (debug profile)."""
    subprocess.run(["cargo", "build", "--locked", "-p", "rustycad-tkdraw", "--bin", "drawexe"],
                   cwd=ROOT, check=True)
    return ROOT / "target" / "debug" / "drawexe"


def run_script(executable: Path, script: Path, environment: dict[str, str], empty_default: Path) -> str:
    """Runs one script in batch mode and returns stdout followed by stderr."""
    env = dict(environment, DRAWDEFAULT=str(empty_default))
    env.pop("CSF_OCCTResourcePath", None)
    limit = f"ulimit -v {MEMORY_LIMIT_KB}; exec \"$0\" -b -f \"$1\""
    result = subprocess.run(["bash", "-c", limit, str(executable), script.name], cwd=script.parent, env=env,
                            capture_output=True, timeout=TIMEOUT_SECONDS)
    return result.stdout.decode(errors="replace") + result.stderr.decode(errors="replace")


def diff(expected: str, actual: str, label: str) -> str:
    """A unified diff of two outputs."""
    lines = difflib.unified_diff(expected.splitlines(True), actual.splitlines(True), "expected", label)
    return "".join(lines)


def check_script(script: Path, native: Path, native_env: dict[str, str], rust: Path, record: bool,
                 empty_default: Path) -> tuple[str, str]:
    """Returns (script name, failure text or '')."""
    native_output = run_script(native, script, native_env, empty_default)
    expected_file = script.with_suffix(".expected")
    failures = []
    if record:
        expected_file.write_text(native_output)
    elif not expected_file.exists() or expected_file.read_text() != native_output:
        recorded = expected_file.read_text() if expected_file.exists() else ""
        failures.append("native output differs from the recorded one:\n" + diff(recorded, native_output, "native"))
    rust_output = run_script(rust, script, dict(os.environ), empty_default)
    if rust_output != native_output:
        failures.append("Rust output differs from native:\n" + diff(native_output, rust_output, "rust"))
    return script.name, "\n".join(failures)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--native-build", required=True, type=Path)
    parser.add_argument("--record", action="store_true")
    parser.add_argument("--jobs", type=int, default=4)
    args = parser.parse_args()
    native = native_executable(args.native_build)
    native_env = native_environment(args.native_build)
    rust = rust_executable()
    scripts = sorted(ORACLE_DIR.glob("*.tcl"))
    with tempfile.TemporaryDirectory() as directory:
        empty_default = Path(directory) / "empty_draw_default.tcl"
        empty_default.write_text("")
        with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as pool:
            jobs = [pool.submit(check_script, script, native, native_env, rust, args.record, empty_default)
                    for script in scripts]
            results = [job.result() for job in jobs]
    failed = [(name, text) for name, text in results if text]
    for name, text in failed:
        print(f"FAIL {name}\n{text}")
    print(f"{len(scripts) - len(failed)}/{len(scripts)} DRAW oracle scripts match native DRAWEXE")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
