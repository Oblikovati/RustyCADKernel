#!/usr/bin/env python3
"""Verify mapped OCCT runtime cases for TKMath, TKG2d, TKG3d, or TKGeomBase."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
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
    require_clean_runtime_git_state,
    require_unchanged_runtime_git_state,
    runtime_input_hashes,
    runtime_git_state,
    require_unchanged_runtime_inputs,
)
from porting import ROOT, snapshot
from legacy_toolkit_sources import (
    SUPPORTED_RUNTIME_TOOLKITS,
    toolkit_rows,
    verify_source_group,
    verify_source_rows,
)
from tkmath_native_provenance import (
    NATIVE_BUILD,
    hash_linked_native_libraries,
    linked_native_library_paths,
    native_configuration,
    native_env,
    native_hashes,
    native_library_hashes,
    verify_native_hashes,
)
from tkernel_mapping import (
    listed_runtime_cases,
    mapped_runtime_sets,
    runtime_case_results,
)


@dataclass(frozen=True)
class VerifiedRun:
    """Pin report inputs before execution; e.g. `VerifiedRun(source, inventory, ...)`."""

    source: Path
    toolkit: str
    inventory: JSONObject
    rows: list[JSONObject]
    sources: set[str]
    input_hashes: dict[str, str]
    rust_state: JSONObject
    cargo_target_dir: Path


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


def native_filter(cases: set[str]) -> str:
    """Build an exact GoogleTest filter; e.g. `native_filter({'Suite.Case'})`."""
    if not cases or any(any(mark in case for mark in ":*?-") for case in cases):
        raise ValueError(
            "native cases must be nonempty and contain no GoogleTest filter metacharacters"
        )
    return ":".join(sorted(cases))


def native_statuses(
    xml_path: Path, expected: set[str], toolkit: str = "TKMath"
) -> dict[str, str]:
    """Require selected cases to pass; e.g. `native_statuses(path, expected, 'TKG2d')`."""
    cases = ET.parse(xml_path).findall(".//testcase")
    identities = [native_identity(case) for case in cases]
    if set(identities) != expected or len(identities) != len(set(identities)):
        raise RuntimeError(
            f"Native {toolkit} runtime cases differ from the mapped exact set"
        )
    require_native_passes(cases, toolkit)
    return {identity: "passed" for identity in identities}


def native_identity(case: ET.Element) -> str:
    """Read one XML identity; e.g. `native_identity(testcase)` returns `Suite.Case`."""
    return case.attrib["classname"] + "." + case.attrib["name"]


def require_native_passes(cases: list[ET.Element], toolkit: str = "TKMath") -> None:
    """Reject failed, unrun, or skipped cases; e.g. `require_native_passes(cases, 'TKG2d')`."""
    if any(
        case.attrib.get("status") != "run" or case.find("failure") is not None
        for case in cases
    ):
        raise RuntimeError(f"Native {toolkit} contains failed or unexecuted cases")
    if any(case.find("skipped") is not None for case in cases):
        raise RuntimeError(
            f"{toolkit} native skips require explicit review before evidence is emitted"
        )
    return None


def rust_statuses(
    expected: set[tuple[str, str, str]], env: dict[str, str], output: Path
) -> set[tuple[str, str, str]]:
    """Run each package/target pair exactly; e.g. `rust_statuses(expected, env, out)`."""
    passed: set[tuple[str, str, str]] = set()
    for package, target in sorted(
        {(package, target) for package, target, _ in expected}
    ):
        passed.update(run_rust_target(package, target, expected, env, output))
    return passed


def run_rust_target(
    package: str,
    target: str,
    expected: set[tuple[str, str, str]],
    env: dict[str, str],
    output: Path,
) -> set[tuple[str, str, str]]:
    """Collect one test binary's passing identities; e.g. `run_rust_target(package, target, pairs, env, out)`."""
    result = rust_target_output(package, target, env, output)
    names = set(re.findall(r"^test (\S+) \.\.\. ok$", result, re.M))
    return validate_rust_target(package, target, names, expected)


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
        output / f"rust-{package}-{target}.log",
        env=env,
        timeout=1800,
    )


def validate_rust_target(
    package: str,
    target: str,
    names: set[str],
    expected: set[tuple[str, str, str]],
) -> set[tuple[str, str, str]]:
    """Require exact package-scoped tests; e.g. `validate_rust_target(package, target, names, expected)`."""
    target_passed = {(package, target, name) for name in names}
    wanted = {
        (name_package, name_target, name)
        for name_package, name_target, name in expected
        if name_package == package and name_target == target
    }
    if target_passed != wanted:
        raise RuntimeError(f"Rust target {target} passed set differs from mappings")
    return target_passed


def run_verification(args: argparse.Namespace) -> JSONObject:
    """Run one toolkit's cases; e.g. `run_verification(args)` for TKMath or TKG2d."""
    source = args.source.resolve()
    inventory = cast(JSONObject, snapshot(source))
    rows = toolkit_rows(ROOT / "porting/parity.json", args.toolkit)
    sources = verify_source_rows(source, rows)
    return execute_verified_rows(args, source, args.toolkit, inventory, rows, sources)


def execute_verified_rows(
    args: argparse.Namespace,
    source: Path,
    toolkit: str,
    inventory: JSONObject,
    rows: list[JSONObject],
    sources: set[str],
) -> JSONObject:
    """Run source-bound mapped suites; e.g. `execute_verified_rows(args, source, inventory, rows, paths)`."""
    rust_state = runtime_git_state(ROOT)
    require_clean_runtime_git_state(rust_state)
    run = VerifiedRun(
        source,
        toolkit,
        inventory,
        rows,
        sources,
        runtime_input_hashes(ROOT),
        rust_state,
        args.cargo_target_dir,
    )
    return execute_verified_run(args, run)


def execute_verified_run(
    args: argparse.Namespace,
    run: VerifiedRun,
) -> JSONObject:
    """Run native and Rust cases; e.g. `execute_verified_run(args, run_snapshot)`."""
    native, provenance = run_native(run.rows, run.source, args)
    rust_passed = run_mapped_rust_cases(
        run.rows, native, args.output, args.cargo_target_dir
    )
    return finish_verified_run(run, native, rust_passed, provenance)


def finish_verified_run(
    run: VerifiedRun,
    native: dict[str, str],
    rust_passed: set[tuple[str, str, str]],
    native_provenance: JSONObject,
) -> JSONObject:
    """Validate run stability and serialize results; e.g. `finish_verified_run(run, native, rust, provenance)`."""
    validate_run_inputs(
        run.input_hashes, run.source, run.inventory, run.rust_state, run.toolkit
    )
    return report(
        run.toolkit,
        run.rows,
        run.inventory,
        run.sources,
        native,
        rust_passed,
        run.input_hashes,
        native_provenance,
        run.rust_state,
        run.cargo_target_dir,
    )


def run_mapped_rust_cases(
    rows: list[JSONObject],
    native: dict[str, str],
    output: Path,
    cargo_target_dir: Path,
) -> set[tuple[str, str, str]]:
    """Run exact Rust peers of native cases; e.g. `run_mapped_rust_cases(rows, native, output)`."""
    expected, rust_expected = mapped_runtime_sets(rows)
    toolkit = str(rows[0]["toolkit"])
    validate_native_identity_set(native, expected, toolkit)
    return rust_statuses(rust_expected, cargo_env(cargo_target_dir), output)


def validate_native_identity_set(
    observed: dict[str, str], expected: set[str], toolkit: str = "TKMath"
) -> None:
    """Reject native case drift; e.g. `validate_native_identity_set(states, cases, 'TKG2d')`."""
    if set(observed) != expected:
        raise RuntimeError(
            f"Mapped {toolkit} native identity set changed during execution"
        )


def validate_run_inputs(
    before: dict[str, str],
    source: Path,
    inventory: JSONObject,
    rust_state: JSONObject,
    toolkit: str = "TKMath",
) -> None:
    """Require unchanged Rust inputs and pinned OCCT; e.g. `validate_run_inputs(before, source, inventory)`."""
    require_unchanged_runtime_inputs(before, runtime_input_hashes(ROOT))
    require_unchanged_runtime_git_state(rust_state, runtime_git_state(ROOT))
    if snapshot(source)["revision"] != inventory["revision"]:
        raise RuntimeError(f"OCCT revision changed while {toolkit} tests were running")


def run_native(
    rows: list[JSONObject], source: Path, args: argparse.Namespace
) -> tuple[dict[str, str], JSONObject]:
    """Build/list/run the mapped OCCT tests; e.g. `run_native(rows, occt, args)`."""
    toolkit = args.toolkit
    cache, executable = prepare_native_binary(args.build_dir, source, args)
    expected, _ = mapped_runtime_sets(rows)
    verify_native_discovery(executable, expected, args.output, toolkit)
    provenance = native_hashes(cache, executable, args.build_dir, source, toolkit)
    statuses = execute_native_filter(executable, expected, args.output, toolkit)
    verify_native_hashes(cache, executable, args.build_dir, provenance, toolkit)
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


def verify_native_discovery(
    executable: Path, expected: set[str], output: Path, toolkit: str = "TKMath"
) -> None:
    """Check exact mapped cases exist; e.g. `verify_native_discovery(exe, cases, out, 'TKG2d')`."""
    listing = command(
        [executable, "--gtest_list_tests"], output / "native-list.log", env=native_env()
    )
    discovered = listed_runtime_cases(listing)
    if not expected <= discovered:
        raise RuntimeError(
            f"Native OpenCascadeGTest lacks mapped {toolkit} case identities"
        )


def execute_native_filter(
    executable: Path, expected: set[str], output: Path, toolkit: str = "TKMath"
) -> dict[str, str]:
    """Run only mapped native cases; e.g. `execute_native_filter(exe, cases, out)`."""
    xml_path = output / f"{toolkit.lower()}-upstream.xml"
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
    return native_statuses(xml_path, expected, toolkit)


def cargo_env(target_dir: Path) -> dict[str, str]:
    """Pin Cargo artifacts to this checkout; e.g. `cargo_env(ROOT / 'target')`."""
    env = native_env()
    cargo_bin = Path.home() / ".cargo/bin"
    env["PATH"] = str(cargo_bin) + os.pathsep + env["PATH"]
    env["CARGO_TARGET_DIR"] = str(target_dir.resolve())
    return env


def report(
    toolkit: str,
    rows: list[JSONObject],
    inventory: JSONObject,
    sources: set[str],
    native: dict[str, str],
    rust_passed: set[tuple[str, str, str]],
    input_hashes: dict[str, str],
    native_provenance: JSONObject,
    rust_state: JSONObject,
    cargo_target_dir: Path,
) -> JSONObject:
    """Serialize observed pairs without asserting toolkit completion; e.g. `report(...)`."""
    return {
        **report_header_fields(
            toolkit,
            inventory,
            input_hashes,
            native_provenance,
            rust_state,
            cargo_target_dir,
        ),
        **runtime_counts_fields(rows, native, rust_passed),
        **runtime_case_fields(rows, native, rust_passed, sources, native_provenance),
    }


def report_header_fields(
    toolkit: str,
    inventory: JSONObject,
    input_hashes: dict[str, str],
    native: JSONObject,
    rust_state: JSONObject,
    cargo_target_dir: Path,
) -> JSONObject:
    """Mark report scope and runtime inputs; e.g. `report_header_fields(inventory, hashes, native)`."""
    return {
        "schema_version": 1,
        "runtime_evidence_schema": 1,
        "runtime_evidence_toolkit": toolkit,
        "revision": inventory["revision"],
        **rust_report_fields(rust_state, cargo_target_dir),
        "source_hashes": input_hashes,
        "native_provenance": native,
        "toolkit_complete": False,
        "scope": f"Mapped {toolkit} cases only; declaration and toolkit acceptance remain pending",
    }


def rust_report_fields(rust_state: JSONObject, cargo_target_dir: Path) -> JSONObject:
    """Record run-time Rust identity and Cargo target; e.g. `rust_report_fields(state, target)`."""
    return {
        "rust_git_commit": rust_state["commit"],
        "rust_source_clean": rust_state["clean"],
        "cargo_target_directory": str(cargo_target_dir.resolve()),
    }


def runtime_counts_fields(
    rows: list[JSONObject],
    native: dict[str, str],
    rust_passed: set[tuple[str, str, str]],
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
    rust_passed: set[tuple[str, str, str]],
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
    """Parse source and toolkit inputs; e.g. `arguments()` defaults to TKMath."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--toolkit", choices=SUPPORTED_RUNTIME_TOOLKITS, default="TKMath"
    )
    parser.add_argument("--source", type=Path, default=ROOT / "../OCCT")
    parser.add_argument("--build-dir", type=Path, default=ROOT / NATIVE_BUILD)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--cargo-target-dir", type=Path, default=ROOT / "target")
    parser.add_argument("--jobs", type=int, default=2)
    args = parser.parse_args()
    if args.output is None:
        args.output = ROOT / f"target/oracle-{args.toolkit.lower()}-legacy"
    return args


def main() -> None:
    """Run the pinned toolkit verifier; e.g. `... --toolkit TKG2d`."""
    args = arguments()
    args.output.mkdir(parents=True, exist_ok=True)
    report_path = args.output / "report.json"
    report_path.unlink(missing_ok=True)
    if sys.platform != "linux":
        raise RuntimeError("This native oracle profile is Linux-only")
    result = run_verification(args)
    report_path.write_text(json.dumps(result, indent=2) + "\n")
    print(f"PASS: {args.toolkit} mapped runtime evidence written to {report_path}")


if __name__ == "__main__":
    main()
