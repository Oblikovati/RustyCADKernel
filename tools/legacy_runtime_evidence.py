#!/usr/bin/env python3
"""Assess whether legacy runtime evidence is fresh and individually joinable."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import subprocess
from typing import cast

from api_inventory.inputs import JSONObject, JSONValue, read_object

ROOT = pathlib.Path(__file__).resolve().parents[1]
DIRECT_HELPERS = (
    "tools/api_inventory_status.py",
    "tools/native_build.py",
    "tools/porting.py",
    "tools/tkernel_native_baseline.py",
    "tools/tkernel_mapping.py",
    "tools/legacy_runtime_evidence.py",
    "tools/tkmath_native_provenance.py",
)
TRANSITIVE_HELPERS = (
    "tools/api_tool_hashes.py",
    "tools/capture_rustdoc_json.py",
    "tools/api_inventory/inputs.py",
)
HASHED_FIXED_INPUTS = (
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
    "tools/verify_tkmath_legacy.py",
    "tools/join_legacy_runtime_evidence.py",
    "tools/tests/test_tkmath_legacy_runtime.py",
    "tools/oracle/tkernel_probe.cxx",
    "tools/oracle/strtod_probe.cxx",
    "tools/verify_strtod.py",
    "tools/verify_ascii_string.py",
    "tools/oracle/ascii_string_probe.cxx",
    "tools/verify_extended_string.py",
    "tools/oracle/extended_string_probe.cxx",
    "tools/verify_tkernel_api.py",
    *DIRECT_HELPERS,
    *TRANSITIVE_HELPERS,
)
AGGREGATE_COUNTS = (
    "legacy_tests_passed",
    "declarations_mapped",
    "rust_translations_passed",
    "rust_runtime_cases_passed",
)


def fixed_runtime_inputs(root: pathlib.Path) -> set[pathlib.Path]:
    """Return fixed verifier inputs; e.g. `fixed_runtime_inputs(Path('/repo'))`."""
    inputs = {root / relative for relative in HASHED_FIXED_INPUTS}
    return inputs


def discovered_runtime_inputs(root: pathlib.Path) -> set[pathlib.Path]:
    """Collect verifier glob inputs; e.g. `discovered_runtime_inputs(Path('/repo'))`."""
    files = {path for path in (root / "crates").rglob("*") if path.is_file()}
    files.update((root / "tools/api_inventory").glob("*.py"))
    files.update((root / "tools/oracle").glob("*.cxx"))
    files.update((root / "tools").glob("tkernel_*_corpus.py"))
    files.update((root / "tools/oracle").glob("*.hxx"))
    return files


def tracked_runtime_inputs(root: pathlib.Path) -> set[str]:
    """Mirror hashed verifier inputs; e.g. `tracked_runtime_inputs(Path('/repo'))`."""
    files = fixed_runtime_inputs(root) | discovered_runtime_inputs(root)
    return {path.relative_to(root).as_posix() for path in files if path.is_file()}


def file_digest(path: pathlib.Path) -> str:
    """Hash an input file; e.g. `file_digest(Path('/repo/Cargo.lock'))`."""
    content = path.read_bytes()
    digest = hashlib.sha256(content).hexdigest()
    return digest


def runtime_input_hashes(root: pathlib.Path) -> dict[str, str]:
    """Capture verifier inputs before execution; e.g. `runtime_input_hashes(root)`."""
    return {
        relative: file_digest(root / relative)
        for relative in sorted(tracked_runtime_inputs(root))
    }


def require_unchanged_runtime_inputs(
    before: dict[str, str], after: dict[str, str]
) -> None:
    """Reject edits during an oracle run; e.g. `require_unchanged_runtime_inputs(a, b)`."""
    changed = sorted(
        relative
        for relative in before.keys() | after.keys()
        if before.get(relative) != after.get(relative)
    )
    if changed:
        raise ValueError(
            f"runtime inputs changed: {changed!r}; expected identical pre/post hashes"
        )


def runtime_git_state(root: pathlib.Path) -> JSONObject:
    """Capture the executing Rust commit and cleanliness; e.g. `runtime_git_state(root)`."""
    return {
        "commit": git_output(root, "rev-parse", "HEAD"),
        "clean": not bool(
            git_output(root, "status", "--porcelain", "--untracked-files=normal")
        ),
    }


def require_clean_runtime_git_state(state: JSONObject) -> None:
    """Require a clean execution tree; e.g. `require_clean_runtime_git_state(state)`."""
    if state.get("clean") is not True:
        raise ValueError("Rust source tree must be clean before runtime verification")


def require_unchanged_runtime_git_state(before: JSONObject, after: JSONObject) -> None:
    """Reject revision or cleanliness drift; e.g. `require_unchanged_runtime_git_state(a, b)`."""
    if before != after:
        raise ValueError("Rust git state changed during runtime verification")
    require_clean_runtime_git_state(after)


def git_output(source: pathlib.Path, *arguments: str) -> str:
    """Read one git value; e.g. `git_output(source, 'rev-parse', 'HEAD')`."""
    command = ["git", "-C", str(source), *arguments]
    return subprocess.run(
        command, check=True, capture_output=True, text=True
    ).stdout.strip()


def source_state(source: pathlib.Path) -> tuple[str | None, bool | None, str | None]:
    """Read OCCT revision and cleanliness; e.g. `source_state(Path('../OCCT'))`."""
    try:
        revision = git_output(source, "rev-parse", "HEAD")
        clean = not git_output(
            source, "status", "--porcelain", "--untracked-files=normal"
        )
    except (OSError, subprocess.CalledProcessError) as error:
        return None, None, f"cannot_inspect_occt_source:{type(error).__name__}"
    return revision, clean, None


def baseline_revision(root: pathlib.Path) -> str | None:
    """Read the repository's pinned OCCT SHA; e.g. `baseline_revision(Path('/repo'))`."""
    try:
        revision = read_object(root / "porting/baseline.json").get("revision")
    except (OSError, ValueError, TypeError):
        return None
    return revision if isinstance(revision, str) and revision else None


def is_digest(value: JSONValue) -> bool:
    """Validate lowercase SHA-256 text; e.g. `is_digest('0' * 64)`."""
    return (
        isinstance(value, str)
        and len(value) == 64
        and all(character in "0123456789abcdef" for character in value)
    )


def report_shape(report: JSONValue) -> tuple[JSONObject | None, list[str]]:
    """Check required aggregate report fields; e.g. `report_shape(report_json)`."""
    if not isinstance(report, dict):
        return None, ["report_not_object"]
    row = cast(JSONObject, report)
    if not isinstance(row.get("revision"), str) or not isinstance(
        row.get("source_hashes"), dict
    ):
        return None, ["missing_revision_or_source_hashes"]
    if not all(is_count(row.get(field)) for field in AGGREGATE_COUNTS):
        return None, ["missing_or_negative_aggregate_counts"]
    return row, []


def is_count(value: JSONValue) -> bool:
    """Accept nonnegative aggregate counts; e.g. `is_count(4)`."""
    if not isinstance(value, int) or isinstance(value, bool):
        return False
    return value >= 0


def valid_hash_table(value: JSONValue) -> bool:
    """Require relative paths mapped to SHA-256; e.g. `valid_hash_table(hashes)`."""
    return isinstance(value, dict) and all(
        isinstance(path, str) and is_digest(digest) for path, digest in value.items()
    )


def source_reasons(
    source: pathlib.Path | None,
    report_revision: str,
    pinned_revision: str | None,
) -> list[str]:
    """Check pin and source checkout; e.g. `source_reasons(path, report_sha, pin)`."""
    if pinned_revision is None:
        return ["occt_pin_unavailable"]
    if report_revision != pinned_revision:
        return ["report_revision_differs_from_pinned_occt"]
    return verify_checkout(source, pinned_revision)


def verify_checkout(source: pathlib.Path | None, pinned: str) -> list[str]:
    """Require a clean pinned checkout; e.g. `verify_checkout(path, revision)`."""
    if source is None:
        return ["occt_checkout_not_verified"]
    actual, clean, issue = source_state(source)
    reasons = [issue] if issue else []
    if actual != pinned and issue is None:
        reasons.append("occt_checkout_revision_mismatch")
    if clean is False:
        reasons.append("occt_source_dirty")
    return [reason for reason in reasons if reason]


def hash_reasons(root: pathlib.Path, recorded: JSONObject) -> list[str]:
    """Compare exact verifier input hashes; e.g. `hash_reasons(root, hashes)`."""
    expected = tracked_runtime_inputs(root)
    missing = sorted(expected - recorded.keys())
    unexpected = sorted(recorded.keys() - expected)
    reasons = missing_hash_reasons(missing, unexpected)
    reasons.extend(changed_hash_reasons(root, expected & recorded.keys(), recorded))
    return reasons


def missing_hash_reasons(missing: list[str], unexpected: list[str]) -> list[str]:
    """Explain input-set drift; e.g. `missing_hash_reasons(['a'], [])`."""
    reasons = []
    if missing:
        reasons.append("runtime_inputs_missing:" + ",".join(missing))
    if unexpected:
        reasons.append("runtime_inputs_unexpected:" + ",".join(unexpected))
    return reasons


def changed_hash_reasons(
    root: pathlib.Path,
    paths: set[str],
    recorded: JSONObject,
) -> list[str]:
    """Find changed or vanished verifier inputs; e.g. `changed_hash_reasons(root, paths, hashes)`."""
    return [
        f"runtime_input_changed:{relative}"
        for relative in sorted(paths)
        if not (root / relative).is_file()
        or file_digest(root / relative) != recorded[relative]
    ]


def granularity_status(report: JSONObject) -> tuple[str, list[str]]:
    """Describe report granularity; e.g. `granularity_status(report)`."""
    if any(
        field in report
        for field in ("cases", "instance_results", "runtime_case_results")
    ):
        return "granular_unjoined", ["declaration_instance_join_not_implemented"]
    return "aggregate_only", ["report_has_no_per_instance_results"]


def assessment(status: str, reasons: list[str]) -> JSONObject:
    """Build non-green assessment; e.g. `assessment('stale', ['changed_hash'])`."""
    return {
        "schema_version": 1,
        "status": status,
        "reasons": reasons,
        "runtime_proof": False,
        "declaration_instance_join": "unavailable",
        "declaration_runtime_status": "unverified",
        "policy": "aggregate verifier counts never prove individual legacy declarations",
    }


def assess_report(
    report: JSONValue,
    root: pathlib.Path,
    occt_source: pathlib.Path | None = None,
) -> JSONObject:
    """Assess freshness without promotion; e.g. `assess_report(report, repo, occt)`."""
    row, errors = report_shape(report)
    if row is None:
        return assessment("invalid", errors)
    return assess_valid_report(row, root, occt_source)


def assess_valid_report(
    report: JSONObject,
    root: pathlib.Path,
    source: pathlib.Path | None,
) -> JSONObject:
    """Assess a shape-checked report; e.g. `assess_valid_report(row, root, source)`."""
    hashes = report["source_hashes"]
    if not valid_hash_table(hashes):
        return assessment("invalid", ["malformed_source_hashes"])
    pin = baseline_revision(root)
    reasons = source_reasons(source, str(report["revision"]), pin)
    reasons.extend(hash_reasons(root, cast(JSONObject, hashes)))
    if reasons:
        return assessment("stale", reasons)
    status, granularity_reasons = granularity_status(report)
    return assessment(status, granularity_reasons)


def load_report(path: pathlib.Path) -> JSONValue:
    """Load verifier JSON; e.g. `load_report(Path('report.json'))`."""
    try:
        return json.loads(path.read_text())
    except (OSError, json.JSONDecodeError):
        return None


def main() -> None:
    """Run the assessor CLI; e.g. `python tools/legacy_runtime_evidence.py report.json`."""
    arguments = parse_arguments()
    result = assess_cli_report(arguments.report, arguments.root, arguments.occt_source)
    rendered = json.dumps(result, indent=2, sort_keys=True) + "\n"
    write_or_print(arguments.output, rendered)


def parse_arguments() -> argparse.Namespace:
    """Parse CLI inputs; e.g. `parse_arguments()` for the command line."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=pathlib.Path)
    parser.add_argument("--root", type=pathlib.Path, default=ROOT)
    parser.add_argument("--occt-source", type=pathlib.Path)
    parser.add_argument("--output", type=pathlib.Path)
    return parser.parse_args()


def assess_cli_report(
    report_path: pathlib.Path,
    root: pathlib.Path,
    source: pathlib.Path | None,
) -> JSONObject:
    """Fail closed for absent files; e.g. `assess_cli_report(path, root, source)`."""
    if not report_path.is_file():
        return assessment("missing", ["report_not_found"])
    return assess_report(load_report(report_path), root.resolve(), source)


def write_or_print(output: pathlib.Path | None, payload: str) -> None:
    """Write JSON or stdout; e.g. `write_or_print(None, '{}\\n')`."""
    if output is not None:
        output.write_text(payload)
        return
    print(payload, end="")


if __name__ == "__main__":
    main()
