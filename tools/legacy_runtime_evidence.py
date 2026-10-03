#!/usr/bin/env python3
"""Assess whether legacy runtime evidence is current and granular enough to trust."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import subprocess
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
REQUIRED_VERIFIER_HELPERS = (
    "tools/api_inventory_status.py",
    "tools/native_build.py",
    "tools/porting.py",
    "tools/tkernel_native_baseline.py",
    "tools/tkernel_mapping.py",
)


def tracked_runtime_inputs(root: pathlib.Path) -> set[str]:
    """Mirror verify_tkernel.py's hashed inputs and include its imported helpers."""
    files = {path for path in (root / "crates").rglob("*") if path.is_file()}
    files.update((root / "tools/oracle").glob("*.cxx"))
    files.update((root / "tools").glob("tkernel_*_corpus.py"))
    files.update((root / "tools/oracle").glob("*.hxx"))
    files.update(
        root / relative
        for relative in (
            "Cargo.toml",
            "Cargo.lock",
            "rust-toolchain.toml",
            "porting/units-data.json",
            "porting/baseline.json",
            "porting/parity.json",
            "tools/verify_geometry.py",
            "tools/verify_units.py",
            "tools/generate_units_data.py",
            "tools/generate_quantity_color_table.py",
            "tools/generate_resource_tables.py",
            "tools/verify_kdtree.py",
            "tools/verify_tkernel.py",
            "tools/oracle/tkernel_probe.cxx",
            "tools/oracle/strtod_probe.cxx",
            "tools/verify_strtod.py",
            "tools/verify_ascii_string.py",
            "tools/oracle/ascii_string_probe.cxx",
            "tools/verify_extended_string.py",
            "tools/oracle/extended_string_probe.cxx",
            "tools/verify_tkernel_api.py",
            *REQUIRED_VERIFIER_HELPERS,
        )
    )
    return {path.relative_to(root).as_posix() for path in files if path.is_file()}


def sha256_file(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git_output(source: pathlib.Path, *args: str) -> str:
    return subprocess.run(
        ["git", "-C", str(source), *args],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()


def source_state(source: pathlib.Path) -> tuple[str | None, bool | None, str | None]:
    """Return OCCT revision, clean state, or a stable diagnostic on git failure."""
    try:
        revision = git_output(source, "rev-parse", "HEAD")
        clean = not git_output(source, "status", "--porcelain")
    except (OSError, subprocess.CalledProcessError) as error:
        return None, None, f"cannot_inspect_occt_source:{type(error).__name__}"
    return revision, clean, None


def assess_report(
    report: Any,
    root: pathlib.Path,
    occt_source: pathlib.Path | None = None,
) -> dict[str, Any]:
    """Return freshness and granularity without promoting aggregate passes to proof."""
    reasons: list[str] = []
    if not isinstance(report, dict):
        return assessment("invalid", ["report_not_object"])
    revision = report.get("revision")
    source_hashes = report.get("source_hashes")
    aggregate_fields = (
        "legacy_tests_passed",
        "declarations_mapped",
        "rust_translations_passed",
        "rust_runtime_cases_passed",
    )
    valid_counts = all(
        isinstance(report.get(field), int) and not isinstance(report.get(field), bool)
        for field in aggregate_fields
    )
    if not isinstance(revision, str) or not isinstance(source_hashes, dict) or not valid_counts:
        return assessment("invalid", ["missing_revision_hashes_or_aggregate_counts"])
    if any(not isinstance(path, str) or not is_digest(digest) for path, digest in source_hashes.items()):
        return assessment("invalid", ["malformed_source_hashes"])

    current_inputs = tracked_runtime_inputs(root)
    missing = sorted(current_inputs - source_hashes.keys())
    unexpected = sorted(source_hashes.keys() - current_inputs)
    if missing:
        reasons.append("runtime_inputs_missing:" + ",".join(missing))
    if unexpected:
        reasons.append("runtime_inputs_unexpected:" + ",".join(unexpected))
    for relative in sorted(current_inputs & source_hashes.keys()):
        path = root / relative
        if not path.is_file() or sha256_file(path) != source_hashes[relative]:
            reasons.append(f"runtime_input_changed:{relative}")

    for relative in REQUIRED_VERIFIER_HELPERS:
        if relative not in source_hashes:
            reasons.append(f"required_verifier_helper_unhashed:{relative}")

    if occt_source is not None:
        actual_revision, clean, problem = source_state(occt_source)
        if problem:
            reasons.append(problem)
        elif actual_revision != revision:
            reasons.append("occt_revision_mismatch")
        if clean is False:
            reasons.append("occt_source_dirty")

    if reasons:
        return assessment("stale", reasons)
    if "cases" in report or "instance_results" in report:
        return assessment("granular_unjoined", ["declaration_instance_join_not_implemented"])
    return assessment("aggregate_only", ["report_has_no_per_instance_results"])


def is_digest(value: Any) -> bool:
    return isinstance(value, str) and len(value) == 64 and all(
        character in "0123456789abcdef" for character in value
    )


def assessment(status: str, reasons: list[str]) -> dict[str, Any]:
    return {
        "schema_version": 1,
        "status": status,
        "reasons": reasons,
        "runtime_proof": False,
        "declaration_instance_join": "unavailable",
        "declaration_runtime_status": "unverified",
        "policy": "aggregate verifier counts never prove individual legacy declarations",
    }


def load_report(path: pathlib.Path) -> Any:
    try:
        return json.loads(path.read_text())
    except (OSError, json.JSONDecodeError):
        return None


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=pathlib.Path)
    parser.add_argument("--root", type=pathlib.Path, default=ROOT)
    parser.add_argument("--occt-source", type=pathlib.Path)
    parser.add_argument("--output", type=pathlib.Path)
    args = parser.parse_args()
    if not args.report.is_file():
        result = assessment("missing", ["report_not_found"])
    else:
        result = assess_report(load_report(args.report), args.root.resolve(), args.occt_source)
    payload = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.write_text(payload)
    else:
        print(payload, end="")


if __name__ == "__main__":
    main()
