#!/usr/bin/env python3
"""Run every tool test module in its own process and report one summary.

One `unittest discover` process keeps the published inventory fixtures of every module in
memory until the end (about 4 GB each for the heavy ones), and the kernel killed it with
OOM on a 109 GB machine while other builds ran. A process per module frees that memory
when the module ends. Example: `python3 tools/run_tool_tests.py --jobs 2`.
"""

from __future__ import annotations

import argparse
import os
import pathlib
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

ROOT = pathlib.Path(__file__).resolve().parents[1]
TESTS = ROOT / "tools/tests"


def test_modules(folder: pathlib.Path, pattern: str) -> list[str]:
    """Sorted test file names; e.g. `test_modules(TESTS, "test_*.py")[0]`."""
    return sorted(path.name for path in folder.glob(pattern))


def run_module(name: str) -> tuple[str, int, str]:
    """Run one module exactly as `discover` would import it; return (name, exit code, tail)."""
    command = [sys.executable, "-m", "unittest", "discover", "-s", str(TESTS), "-p", name]
    environment = dict(os.environ, PYTHONPATH=str(ROOT / "tools"))
    result = subprocess.run(command, cwd=ROOT, env=environment, capture_output=True, text=True)
    return name, result.returncode, result.stderr[-4000:]


def failed_modules(results: list[tuple[str, int, str]]) -> list[tuple[str, int, str]]:
    """Results whose process did not exit cleanly (failure, error or kill)."""
    return [result for result in results if result[1] != 0]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--jobs", type=int, default=2)
    parser.add_argument("--pattern", default="test_*.py")
    args = parser.parse_args()
    names = test_modules(TESTS, args.pattern)
    with ThreadPoolExecutor(max_workers=max(1, args.jobs)) as pool:
        results = list(pool.map(run_module, names))
    failures = failed_modules(results)
    for name, code, tail in failures:
        print(f"FAILED {name} (exit {code})\n{tail}")
    print(f"tool test modules: {len(names) - len(failures)} passed, {len(failures)} failed")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
