#!/usr/bin/env python3
"""Verify mapped TKMath legacy tests individually against OCCT and Rust."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import xml.etree.ElementTree as ET
from typing import cast

from api_inventory.inputs import JSONObject
from legacy_runtime_evidence import (
    runtime_input_hashes,
    require_unchanged_runtime_inputs,
)
from porting import ROOT, snapshot
from tkernel_mapping import (
    listed_runtime_cases,
    mapped_runtime_sets,
    runtime_case_results,
)

NATIVE_BUILD = Path("target/occt-modeling-full")
NATIVE_EXECUTABLE = Path("lin64/gcc/bini/OpenCascadeGTest")
DECLARATION = re.compile(r"(?:TEST(?:_F)?|TYPED_TEST)\((\w+), (\w+)\)")


def command(
    arguments: list[str | Path], log: Path, *, env: dict[str, str], timeout: int = 900
) -> str:
    """Run a captured command; e.g. `command(['cargo','--version'], log, env=env)`."""
    result = subprocess.run(
        [str(argument) for argument in arguments],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        timeout=timeout,
    )
    log.parent.mkdir(parents=True, exist_ok=True)
    log.write_text(result.stdout + result.stderr)
    if result.returncode:
        raise RuntimeError(f"Command failed ({result.returncode}); see {log}")
    return result.stdout


def toolkit_rows(ledger_path: Path) -> list[JSONObject]:
    """Load translated TKMath declarations; e.g. `toolkit_rows(ROOT / 'porting/parity.json')`."""
    ledger = json.loads(ledger_path.read_text())
    rows = [row for row in ledger["tests"] if row["toolkit"] == "TKMath"]
    if not rows or any(row["status"] != "translated" for row in rows):
        raise RuntimeError("TKMath mappings must be nonempty and translated")
    return rows


def verify_source_rows(source: Path, rows: list[JSONObject]) -> set[str]:
    """Check hashes and complete suite declarations; e.g. `verify_source_rows(occt, rows)`."""
    grouped: dict[str, list[JSONObject]] = {}
    for row in rows:
        grouped.setdefault(str(row["source"]), []).append(row)
    for relative, mappings in grouped.items():
        verify_source_group(source, relative, mappings)
    return set(grouped)


def verify_source_group(
    source: Path, relative: str, mappings: list[JSONObject]
) -> None:
    """Reject incomplete or stale source mappings; e.g. `verify_source_group(root, path, rows)`."""
    contents = (source / relative).read_bytes()
    digest = hashlib.sha256(contents).hexdigest()
    names = {
        f"{suite}.{case}" for suite, case in DECLARATION.findall(contents.decode())
    }
    mapped = {str(row["upstream_id"]) for row in mappings}
    if names != mapped or any(row["source_sha256"] != digest for row in mappings):
        raise RuntimeError(f"Stale or incomplete TKMath source mapping: {relative}")


def native_configuration(build: Path, source: Path) -> tuple[Path, Path]:
    """Require a CMake test build tied to OCCT; e.g. `native_configuration(build, source)`."""
    cache = build / "CMakeCache.txt"
    contents = cache.read_text()
    match = re.search(r"^CMAKE_HOME_DIRECTORY:INTERNAL=(.*)$", contents, re.M)
    if match is None or Path(match.group(1)).resolve() != source:
        raise RuntimeError(f"{cache}: expected CMake home directory {source}")
    executable = build / NATIVE_EXECUTABLE
    if not executable.is_file():
        raise RuntimeError(f"{executable}: expected OpenCascadeGTest executable")
    return cache, executable


def sha256(path: Path) -> str:
    """Hash one provenance input; e.g. `sha256(build / 'CMakeCache.txt')`."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def native_filter(cases: set[str]) -> str:
    """Build an exact GoogleTest filter; e.g. `native_filter({'Suite.Case'})`."""
    if not cases or any(any(mark in case for mark in ":*?-") for case in cases):
        raise ValueError(
            "native cases must be nonempty and contain no GoogleTest filter metacharacters"
        )
    return ":".join(sorted(cases))


def native_statuses(xml_path: Path, expected: set[str]) -> dict[str, str]:
    """Require every selected native case to pass; e.g. `native_statuses(path, expected)`."""
    cases = ET.parse(xml_path).findall(".//testcase")
    identities = [native_identity(case) for case in cases]
    if set(identities) != expected or len(identities) != len(set(identities)):
        raise RuntimeError(
            "Native TKMath runtime cases differ from the mapped exact set"
        )
    require_native_passes(cases)
    return {identity: "passed" for identity in identities}


def native_identity(case: ET.Element) -> str:
    """Read one XML identity; e.g. `native_identity(testcase)` returns `Suite.Case`."""
    return case.attrib["classname"] + "." + case.attrib["name"]


def require_native_passes(cases: list[ET.Element]) -> None:
    """Reject failed, unrun or skipped cases; e.g. `require_native_passes(xml_cases)`."""
    if any(
        case.attrib.get("status") != "run" or case.find("failure") is not None
        for case in cases
    ):
        raise RuntimeError("Native TKMath contains failed or unexecuted cases")
    if any(case.find("skipped") is not None for case in cases):
        raise RuntimeError(
            "TKMath native skips require explicit review before evidence is emitted"
        )
    return None


def rust_statuses(
    package: str, expected: set[tuple[str, str]], env: dict[str, str], output: Path
) -> set[tuple[str, str]]:
    """Run exact mapped Cargo targets; e.g. `rust_statuses('rustycad-tkmath', expected, env, out)`."""
    passed: set[tuple[str, str]] = set()
    for target in sorted({target for target, _ in expected}):
        passed.update(run_rust_target(package, target, expected, env, output))
    return passed


def run_rust_target(
    package: str,
    target: str,
    expected: set[tuple[str, str]],
    env: dict[str, str],
    output: Path,
) -> set[tuple[str, str]]:
    """Collect one test binary's passing identities; e.g. `run_rust_target(package, target, pairs, env, out)`."""
    result = rust_target_output(package, target, env, output)
    names = set(re.findall(r"^test (\S+) \.\.\. ok$", result, re.M))
    return validate_rust_target(target, names, expected)


def rust_target_output(
    package: str, target: str, env: dict[str, str], output: Path
) -> str:
    """Run one mapped Cargo test binary; e.g. `rust_target_output(package, target, env, out)`."""
    return command(
        [
            "cargo",
            "test",
            "--locked",
            "-p",
            package,
            "--test",
            target,
            "--",
            "--test-threads=1",
        ],
        output / f"rust-{target}.log",
        env=env,
        timeout=1800,
    )


def validate_rust_target(
    target: str, names: set[str], expected: set[tuple[str, str]]
) -> set[tuple[str, str]]:
    """Require exact mapped Rust test events; e.g. `validate_rust_target(target, names, pairs)`."""
    target_passed = {(target, name) for name in names}
    wanted = {
        (name_target, name) for name_target, name in expected if name_target == target
    }
    if target_passed != wanted:
        raise RuntimeError(f"Rust target {target} passed set differs from mappings")
    return target_passed


def package_name(rows: list[dict[str, object]]) -> str:
    """Require one Rust package for this toolkit; e.g. `package_name(tkmath_rows)`."""
    packages = {str(row["rust_package"]) for row in rows}
    if len(packages) != 1:
        raise RuntimeError(
            f"TKMath mappings use unexpected Rust packages: {sorted(packages)}"
        )
    return packages.pop()


def run_verification(args: argparse.Namespace) -> JSONObject:
    """Run native and Rust cases and assemble exact pair evidence; e.g. `run_verification(args)`."""
    source = args.source.resolve()
    inventory = cast(JSONObject, snapshot(source))
    rows = toolkit_rows(ROOT / "porting/parity.json")
    sources = verify_source_rows(source, rows)
    return execute_verified_rows(args, source, inventory, rows, sources)


def execute_verified_rows(
    args: argparse.Namespace,
    source: Path,
    inventory: JSONObject,
    rows: list[JSONObject],
    sources: set[str],
) -> JSONObject:
    """Run source-bound mapped suites; e.g. `execute_verified_rows(args, source, inventory, rows, paths)`."""
    before = runtime_input_hashes(ROOT)
    native, native_provenance = run_native(rows, source, args)
    expected, rust_expected = mapped_runtime_sets(rows)
    validate_native_identity_set(native, expected)
    rust_passed = rust_statuses(
        package_name(rows), rust_expected, cargo_env(), args.output
    )
    validate_run_inputs(before, source, inventory)
    return report(
        rows, inventory, sources, native, rust_passed, before, native_provenance
    )


def validate_native_identity_set(observed: dict[str, str], expected: set[str]) -> None:
    """Reject native case drift; e.g. `validate_native_identity_set(statuses, expected)`."""
    if set(observed) != expected:
        raise RuntimeError("Mapped TKMath native identity set changed during execution")


def validate_run_inputs(
    before: dict[str, str], source: Path, inventory: JSONObject
) -> None:
    """Require unchanged Rust inputs and pinned OCCT; e.g. `validate_run_inputs(before, source, inventory)`."""
    require_unchanged_runtime_inputs(before, runtime_input_hashes(ROOT))
    if snapshot(source)["revision"] != inventory["revision"]:
        raise RuntimeError("OCCT revision changed while TKMath tests were running")


def run_native(
    rows: list[JSONObject], source: Path, args: argparse.Namespace
) -> tuple[dict[str, str], JSONObject]:
    """Build/list/run the mapped OCCT tests; e.g. `run_native(rows, occt, args)`."""
    cache, executable = prepare_native_binary(args.build_dir, source, args)
    expected, _ = mapped_runtime_sets(rows)
    verify_native_discovery(executable, expected, args.output)
    provenance = native_hashes(cache, executable, args.build_dir, source)
    statuses = execute_native_filter(executable, expected, args.output)
    verify_native_hashes(cache, executable, provenance)
    return statuses, provenance


def prepare_native_binary(
    build: Path, source: Path, args: argparse.Namespace
) -> tuple[Path, Path]:
    """Rebuild the pinned GoogleTest target; e.g. `prepare_native_binary(build, source, args)`."""
    cache, executable = native_configuration(build, source)
    command(
        [
            "cmake",
            "--build",
            args.build_dir,
            "--target",
            "OpenCascadeGTest",
            "--parallel",
            str(args.jobs),
        ],
        args.output / "native-build.log",
        env=native_env(),
        timeout=3600,
    )
    return native_configuration(build, source)


def verify_native_discovery(executable: Path, expected: set[str], output: Path) -> None:
    """Check every exact mapped case exists in the built binary; e.g. `verify_native_discovery(exe, cases, out)`."""
    listing = command(
        [executable, "--gtest_list_tests"], output / "native-list.log", env=native_env()
    )
    discovered = listed_runtime_cases(listing)
    if not expected <= discovered:
        raise RuntimeError(
            "Native OpenCascadeGTest lacks mapped TKMath case identities"
        )


def native_hashes(
    cache: Path, executable: Path, build: Path, source: Path
) -> JSONObject:
    """Fingerprint the native test build; e.g. `native_hashes(cache, exe, build, source)`."""
    return {
        "build_directory": str(build.resolve()),
        "cmake_cache_sha256": sha256(cache),
        "cmake_home_directory": str(source),
        "executable": str(executable.resolve()),
        "executable_sha256": sha256(executable),
    }


def execute_native_filter(
    executable: Path, expected: set[str], output: Path
) -> dict[str, str]:
    """Run only mapped native cases; e.g. `execute_native_filter(exe, cases, out)`."""
    xml_path = output / "tkmath-upstream.xml"
    filter_value = native_filter(expected)
    command(
        [
            executable,
            f"--gtest_filter={filter_value}",
            f"--gtest_output=xml:{xml_path}",
        ],
        output / "native-run.log",
        env=native_env(),
        timeout=3600,
    )
    return native_statuses(xml_path, expected)


def verify_native_hashes(cache: Path, executable: Path, before: JSONObject) -> None:
    """Ensure the build identity stayed fixed; e.g. `verify_native_hashes(cache, exe, provenance)`."""
    if (
        sha256(executable) != before["executable_sha256"]
        or sha256(cache) != before["cmake_cache_sha256"]
    ):
        raise RuntimeError(
            "Native test binary or CMake provenance changed during the run"
        )


def native_env() -> dict[str, str]:
    """Set the native oracle's deterministic C locale; e.g. `native_env()`."""
    return dict(os.environ, LC_ALL="C", LANG="C")


def cargo_env() -> dict[str, str]:
    """Expose the pinned rustup toolchain; e.g. `cargo_env()`."""
    env = native_env()
    cargo_bin = Path.home() / ".cargo/bin"
    env["PATH"] = str(cargo_bin) + os.pathsep + env["PATH"]
    return env


def report(
    rows: list[JSONObject],
    inventory: JSONObject,
    sources: set[str],
    native: dict[str, str],
    rust_passed: set[tuple[str, str]],
    input_hashes: dict[str, str],
    native_provenance: JSONObject,
) -> JSONObject:
    """Serialize observed pairs without asserting toolkit completion; e.g. `report(...)`."""
    return {
        **report_header_fields(inventory, input_hashes, native_provenance),
        **runtime_counts_fields(rows, native, rust_passed),
        **runtime_case_fields(rows, native, rust_passed, sources, native_provenance),
    }


def report_header_fields(
    inventory: JSONObject, input_hashes: dict[str, str], native: JSONObject
) -> JSONObject:
    """Mark report scope and runtime inputs; e.g. `report_header_fields(inventory, hashes, native)`."""
    return {
        "schema_version": 1,
        "runtime_evidence_schema": 1,
        "runtime_evidence_toolkit": "TKMath",
        "revision": inventory["revision"],
        "source_hashes": input_hashes,
        "native_provenance": native,
        "toolkit_complete": False,
        "scope": "Mapped TKMath cases only; declaration and toolkit acceptance remain pending",
    }


def runtime_counts_fields(
    rows: list[JSONObject],
    native: dict[str, str],
    rust_passed: set[tuple[str, str]],
) -> JSONObject:
    """Record aggregate counts beside per-case rows; e.g. `runtime_counts_fields(rows, native, rust)`."""
    return {
        "declarations_mapped": len(rows),
        "legacy_tests_passed": len(native),
        "rust_translations_passed": len(rows),
        "rust_runtime_cases_passed": len(rust_passed),
        "expected_rust_case_count": len(mapped_runtime_sets(rows)[1]),
    }


def runtime_case_fields(
    rows: list[JSONObject],
    native: dict[str, str],
    rust_passed: set[tuple[str, str]],
    sources: set[str],
    native_provenance: JSONObject,
) -> JSONObject:
    """Store exact runtime identities and native executable hash; e.g. `runtime_case_fields(...)`."""
    return {
        "runtime_case_results": runtime_case_results(rows, native, rust_passed),
        "mapped_source_paths": sorted(sources),
        "native_binary_sha256": native_provenance["executable_sha256"],
    }


def arguments() -> argparse.Namespace:
    """Parse source and build inputs; e.g. `arguments()` for the verifier CLI."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=ROOT / "../OCCT")
    parser.add_argument("--build-dir", type=Path, default=ROOT / NATIVE_BUILD)
    parser.add_argument(
        "--output", type=Path, default=ROOT / "target/oracle-tkmath-legacy"
    )
    parser.add_argument("--jobs", type=int, default=2)
    return parser.parse_args()


def main() -> None:
    """Run the pinned TKMath runtime verifier; e.g. `python tools/verify_tkmath_legacy.py`."""
    if sys.platform != "linux":
        raise RuntimeError("This native oracle profile is Linux-only")
    args = arguments()
    args.output.mkdir(parents=True, exist_ok=True)
    result = run_verification(args)
    report_path = args.output / "report.json"
    report_path.write_text(json.dumps(result, indent=2) + "\n")
    print(f"PASS: TKMath mapped runtime evidence written to {report_path}")


if __name__ == "__main__":
    main()
