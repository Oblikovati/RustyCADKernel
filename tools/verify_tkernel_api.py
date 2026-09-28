#!/usr/bin/env python3
"""Compare the ported TKernel API surface with the pinned OCCT kernel, request by request.

Usage:
    python3 tools/verify_tkernel_api.py --source /path/to/OCCT \
        [--native-build target/occt-tkernel-full] [--only exception_]

The native probe (tools/oracle/tkernel_api_probe.cxx) and its Rust twin
(crates/tkernel/examples/api_probe.rs) read the same request lines and must print
byte-identical responses. Each area contributes a `tools/tkernel_<area>_corpus.py`.
Success evidence is written only after every response agrees.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

from porting import ROOT, snapshot
from tkernel_ascii_string_api_corpus import append_ascii_string_api_requests
from tkernel_exceptions_corpus import append_exceptions_requests
from tkernel_extended_string_api_corpus import append_extended_string_api_requests
from tkernel_standard_runtime_corpus import append_standard_runtime_requests
from tkernel_standard_services_corpus import append_standard_services_requests
from tkernel_standard_values_corpus import append_standard_values_requests

APPENDERS = [append_exceptions_requests, append_ascii_string_api_requests, append_extended_string_api_requests,
             append_standard_values_requests, append_standard_runtime_requests, append_standard_services_requests]


def corpus(prefix: str = "") -> str:
    """Every area's requests, optionally only operations starting with `prefix`; corpus('guid')."""
    requests: list[str] = []
    for append in APPENDERS:
        append(requests)
    return "".join(f"{line}\n" for line in requests if line.startswith(prefix))


def run(command: list[str], log: Path, requests: str | None = None) -> str:
    """Run a checked command in the C locale, keeping its output in `log`; run(cmd, log)."""
    environment = dict(os.environ, LC_ALL="C", LANG="C")
    environment["PATH"] = str(Path.home() / ".cargo/bin") + os.pathsep + environment["PATH"]
    result = subprocess.run(command, input=requests, text=True, capture_output=True, cwd=ROOT,
                            env=environment, check=False)
    log.write_text(result.stdout + result.stderr)
    if result.returncode:
        raise RuntimeError(f"command failed ({result.returncode}): {' '.join(command)}; see {log}")
    return result.stdout


def build_native(build: Path, output: Path) -> Path:
    """Compile the native probe against the prebuilt TKernel; build_native(build_dir, out_dir)."""
    libraries = sorted(build.rglob("libTKernel.so"))
    if len(libraries) != 1:
        raise RuntimeError(f"expected exactly one libTKernel.so under {build}, found {len(libraries)}")
    library = libraries[0]
    executable = output / "native-probe"
    run(["c++", "-std=c++17", "-O2", "-pthread", f"-I{build / 'include/opencascade'}",
         str(ROOT / "tools/oracle/tkernel_api_probe.cxx"), str(library), f"-Wl,-rpath,{library.parent}",
         "-o", str(executable)], output / "compile.log")
    return executable


def compare(requests: str, native: str, rust: str) -> None:
    """Require one identical response per request; name the first differing request."""
    lines = requests.splitlines()
    if len(native.splitlines()) != len(lines):
        raise RuntimeError(f"native probe answered {len(native.splitlines())} of {len(lines)} requests")
    for request, expected, actual in zip(lines, native.splitlines(), rust.splitlines()):
        if expected != actual:
            raise RuntimeError(f"mismatch for {request!r}: OCCT {expected!r}; Rust {actual!r}")
    if native != rust:
        raise RuntimeError(f"Rust probe answered {len(rust.splitlines())} of {len(lines)} requests")


def main() -> None:
    """Build both probes, compare every request and write the report on success."""
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--source", type=Path, default=ROOT / "../OCCT")
    parser.add_argument("--native-build", type=Path, default=ROOT / "target/occt-tkernel-full")
    parser.add_argument("--only", default="", help="compare only operations with this prefix")
    args = parser.parse_args()
    inventory = snapshot(args.source.resolve())
    output = ROOT / "target/oracle-tkernel/api"
    output.mkdir(parents=True, exist_ok=True)
    (output / "report.json").unlink(missing_ok=True)
    executable = build_native(args.native_build.resolve(), output)
    run(["cargo", "build", "--locked", "-p", "rustycad-tkernel", "--example", "api_probe"], output / "rust-build.log")
    requests = corpus(args.only)
    (output / "inputs.txt").write_text(requests)
    native = run([str(executable)], output / "native.txt", requests)
    rust = run([str(ROOT / "target/debug/examples/api_probe")], output / "rust.txt", requests)
    compare(requests, native, rust)
    report = {"revision": inventory["revision"], "cases_passed": len(requests.splitlines()), "only": args.only,
              "input_sha256": hashlib.sha256(requests.encode()).hexdigest(),
              "output_sha256": hashlib.sha256(native.encode()).hexdigest()}
    (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(f"TKernel API: {report['cases_passed']} exact-output comparisons passed")


if __name__ == "__main__":
    try:
        main()
    except (RuntimeError, ValueError, OSError) as error:
        print(f"error: {error}")
        raise SystemExit(1) from error
