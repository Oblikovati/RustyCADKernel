#!/usr/bin/env python3
"""Join per-case verifier results to declarations without promoting skips or gaps."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import re
from typing import cast

from api_inventory.inputs import (
    JSONObject,
    JSONValue,
    object_rows,
    read_object,
    text_field,
)
from legacy_runtime_evidence import assess_report, is_digest
from publish_legacy_test_inventory import gtest_records
from porting import snapshot
from tkernel_mapping import runtime_pairs

ROOT = pathlib.Path(__file__).resolve().parents[1]
NATIVE_STATES = {"passed", "upstream_skipped"}


def case_identity(row: JSONObject) -> tuple[str, str, str, str, str, str]:
    """Key a case by declaration and both runtime names; e.g. `case_identity(row)`."""
    return tuple(
        string_field(row, field)
        for field in (
            "toolkit",
            "source",
            "upstream_id",
            "native_case",
            "rust_target",
            "rust_test",
        )
    )


def string_field(row: JSONObject, field: str) -> str:
    """Require a string identity component; e.g. `string_field(row, 'source')`."""
    value = row.get(field)
    if not isinstance(value, str) or not value:
        raise ValueError(f"{field}: expected nonempty string, got {value!r}")
    return value


def case_rows(value: JSONValue) -> list[JSONObject]:
    """Validate case result records; e.g. `case_rows(report['runtime_case_results'])`."""
    if not isinstance(value, list) or not all(isinstance(row, dict) for row in value):
        raise ValueError("runtime_case_results: expected array of objects")
    rows = cast(list[JSONObject], value)
    validate_case_rows(rows)
    return rows


def validate_case_rows(rows: list[JSONObject]) -> None:
    """Reject malformed status rows; e.g. `validate_case_rows(rows)`."""
    for row in rows:
        case_identity(row)
        if not is_digest(row.get("source_sha256")):
            raise ValueError("source_sha256: expected lowercase SHA-256 digest")
        if row.get("native_status") not in NATIVE_STATES:
            raise ValueError(
                f"native_status: unexpected value {row.get('native_status')!r}"
            )
        if row.get("rust_status") not in {"passed", "missing"}:
            raise ValueError(
                f"rust_status: unexpected value {row.get('rust_status')!r}"
            )


def case_index(
    rows: list[JSONObject],
) -> dict[tuple[str, str, str, str, str, str], JSONObject]:
    """Index unique pair identities; e.g. `case_index(runtime_case_rows)`."""
    index: dict[tuple[str, str, str, str, str, str], JSONObject] = {}
    for row in rows:
        key = case_identity(row)
        if key in index:
            raise ValueError(f"duplicate runtime case identity: {key!r}")
        index[key] = row
    return index


def expected_case_keys(
    declaration: JSONObject, mapping: JSONObject
) -> list[tuple[str, str, str, str, str, str]]:
    """Expand a mapping into declaration-bound cases; e.g. `expected_case_keys(row, map)`."""
    base = (
        string_field(declaration, "toolkit"),
        string_field(declaration, "source"),
        string_field(declaration, "name"),
    )
    target = string_field(mapping, "rust_target")
    return [(*base, native, target, rust) for native, rust in runtime_pairs(mapping)]


def declaration_evidence(
    declaration: JSONObject,
    index: dict[tuple[str, str, str, str, str, str], JSONObject],
    consumed: set[tuple[str, str, str, str, str, str]],
    toolkit_scope: str | None,
) -> JSONObject:
    """Attach exact passing evidence; e.g. `declaration_evidence(row, index, consumed)`."""
    reason = mapping_unverified_reason(declaration, toolkit_scope)
    if reason:
        return with_runtime_state(declaration, "unverified", [], reason)
    mapping = declaration.get("mapped_rust_test")
    keys = expected_case_keys(declaration, cast(JSONObject, mapping))
    observed = collect_expected_cases(keys, index, consumed)
    if not source_digests_match(cast(JSONObject, mapping), observed):
        return with_runtime_state(
            declaration, "unverified", observed, "source_digest_mismatch"
        )
    return evidence_for_mapping(declaration, cast(JSONObject, mapping), keys, observed)


def mapping_unverified_reason(declaration: JSONObject, scope: str | None) -> str:
    """Explain declaration gaps; e.g. `mapping_unverified_reason(row, 'TKernel')`."""
    if scope and declaration.get("toolkit") != scope:
        return "outside_runtime_toolkit_scope"
    if declaration.get("mapping_status") != "mapped":
        return f"mapping_status:{declaration.get('mapping_status', 'unknown')}"
    return (
        ""
        if isinstance(declaration.get("mapped_rust_test"), dict)
        else "mapping_row_missing"
    )


def collect_expected_cases(
    keys: list[tuple[str, str, str, str, str, str]],
    index: dict[tuple[str, str, str, str, str, str], JSONObject],
    consumed: set[tuple[str, str, str, str, str, str]],
) -> list[JSONObject]:
    """Collect exact observed case pairs; e.g. `collect_expected_cases(keys, index, consumed)`."""
    observed = [index[key] for key in keys if key in index]
    consumed.update(key for key in keys if key in index)
    return observed


def source_digests_match(mapping: JSONObject, cases: list[JSONObject]) -> bool:
    """Require case SHA to match parity; e.g. `source_digests_match(mapping, cases)`."""
    expected = mapping.get("source_sha256")
    return all(case.get("source_sha256") == expected for case in cases)


def with_runtime_state(
    declaration: JSONObject, status: str, cases: list[JSONObject], reason: str = ""
) -> JSONObject:
    """Copy a declaration with explicit runtime state; e.g. `with_runtime_state(row, 'unverified', [])`."""
    result = dict(declaration)
    result["runtime_status"] = status
    result["runtime_reason"] = reason
    result["runtime_cases"] = cases
    result["legacy_assertions_passed"] = status == "passed"
    return cast(JSONObject, result)


def evidence_for_mapping(
    declaration: JSONObject,
    mapping: JSONObject,
    keys: list[tuple[str, str, str, str, str, str]],
    observed: list[JSONObject],
) -> JSONObject:
    """Classify all paired results; e.g. `evidence_for_mapping(row, map, keys, results)`."""
    if len(observed) != len(keys) or not keys:
        return with_runtime_state(
            declaration, "unverified", observed, "missing_runtime_case_result"
        )
    if mapping.get("status") == "upstream_skipped":
        return skipped_evidence(declaration, observed)
    if all_pairs_passed(observed):
        return with_runtime_state(declaration, "passed", observed)
    return with_runtime_state(
        declaration, "unverified", observed, "runtime_pair_not_passed"
    )


def all_pairs_passed(cases: list[JSONObject]) -> bool:
    """Require native and Rust pass states; e.g. `all_pairs_passed(cases)`."""
    return all(
        case["native_status"] == "passed" and case["rust_status"] == "passed"
        for case in cases
    )


def skipped_evidence(declaration: JSONObject, cases: list[JSONObject]) -> JSONObject:
    """Keep intentional upstream skips non-green; e.g. `skipped_evidence(row, cases)`."""
    only_skipped = all(
        case["native_status"] == "upstream_skipped" and case["rust_status"] == "passed"
        for case in cases
    )
    state = "upstream_skipped" if only_skipped else "unverified"
    result = with_runtime_state(
        declaration, state, cases, "native legacy test is an upstream skip"
    )
    result["legacy_assertions_passed"] = False
    return result


def join_declarations(
    declarations: list[JSONObject],
    case_results: list[JSONObject],
    toolkit_scope: str | None = None,
) -> tuple[list[JSONObject], list[JSONObject]]:
    """Join mappings and results without name-only matching; e.g. `join_declarations(rows, cases)`."""
    index = case_index(case_results)
    consumed: set[tuple[str, str, str, str, str, str]] = set()
    joined = [
        declaration_evidence(row, index, consumed, toolkit_scope)
        for row in declarations
    ]
    unclaimed = [row for key, row in index.items() if key not in consumed]
    return joined, unclaimed


def join_report(
    report: JSONObject, inventory: JSONObject, parity: JSONObject
) -> JSONObject:
    """Produce declaration-level results; e.g. `join_report(report, inventory, parity)`."""
    declarations, anomalies = gtest_records(inventory, parity)
    case_values = report.get("runtime_case_results")
    cases = case_rows(case_values) if case_values is not None else []
    scope = string_field(report, "runtime_evidence_toolkit")
    joined, unclaimed = join_declarations(declarations, cases, scope)
    return joined_report(report, inventory, joined, anomalies, unclaimed)


def joined_report(
    report: JSONObject,
    inventory: JSONObject,
    rows: list[JSONObject],
    anomalies: list[JSONObject],
    unclaimed: list[JSONObject],
) -> JSONObject:
    """Summarize an incremental join; e.g. `joined_report(report, inventory, rows, [], [])`."""
    return {
        "schema_version": 1,
        "occt_revision": report["revision"],
        "declarations": rows,
        "mapping_anomalies": anomalies,
        "unclaimed_runtime_cases": unclaimed,
        "counts": runtime_counts(rows),
        "toolkit_proofs": toolkit_proofs(rows),
        **global_proof_fields(),
        "inventory_revision_matches_report": inventory.get("revision")
        == report["revision"],
    }


def global_proof_fields() -> JSONObject:
    """Declare incomplete acceptance; e.g. `global_proof_fields()`."""
    return {
        "runtime_proof": False,
        "global_proof_reason": "runtime report covers mapped suites only; full legacy obligations remain",
    }


def runtime_counts(rows: list[JSONObject]) -> JSONObject:
    """Count per-declaration statuses; e.g. `runtime_counts(joined_rows)`."""
    passed = sum(row.get("runtime_status") == "passed" for row in rows)
    skipped = sum(row.get("runtime_status") == "upstream_skipped" for row in rows)
    return {
        "declared": len(rows),
        "passed": passed,
        "upstream_skipped": skipped,
        "unverified": len(rows) - passed - skipped,
    }


def toolkit_proofs(rows: list[JSONObject]) -> JSONObject:
    """Keep global completion false while reporting incremental counts; e.g. `toolkit_proofs(rows)`."""
    toolkits = sorted({string_field(row, "toolkit") for row in rows})
    return {
        toolkit: {
            "runtime_proof": False,
            "passed": sum(
                row.get("toolkit") == toolkit and row.get("runtime_status") == "passed"
                for row in rows
            ),
        }
        for toolkit in toolkits
    }


def checked_report(
    report: JSONObject,
    root: pathlib.Path,
    source: pathlib.Path,
    parity: JSONObject,
) -> JSONObject:
    """Require fresh source-bound evidence; e.g. `checked_report(report, root, source, parity)`."""
    freshness = assess_report(report, root, source)
    if freshness["status"] == "stale" or freshness["status"] == "invalid":
        raise ValueError(
            f"runtime report is {freshness['status']}: {freshness['reasons']}"
        )
    inventory = snapshot(source)
    if report.get("revision") != inventory.get("revision"):
        raise ValueError("inventory revision does not match runtime report")
    source_errors = source_mapping_errors(parity, source)
    if source_errors:
        raise ValueError(f"stale parity source hashes: {source_errors}")
    require_schema(report)
    return join_report(report, inventory, parity)


def source_mapping_errors(parity: JSONObject, source: pathlib.Path) -> list[str]:
    """Check mapped C++ source hashes against pinned OCCT; e.g. `source_mapping_errors(parity, source)`."""
    errors = []
    for row in object_rows(parity.get("tests"), "parity tests"):
        if text_field(row, "toolkit") != "TKernel":
            continue
        errors.extend(one_source_mapping_error(row, source))
    return errors


def one_source_mapping_error(row: JSONObject, source: pathlib.Path) -> list[str]:
    """Verify one mapped test file; e.g. `one_source_mapping_error(row, source)`."""
    relative = text_field(row, "source")
    path = source / relative
    expected = row.get("source_sha256")
    if not is_digest(expected) or not path.is_file():
        return [f"{relative}: missing file or source digest"]
    actual = hashlib.sha256(path.read_bytes()).hexdigest()
    return [] if actual == expected else [f"{relative}: source SHA mismatch"]


def require_schema(report: JSONObject) -> None:
    """Require per-case schema version 1; e.g. `require_schema(report)`."""
    schema = report.get("runtime_evidence_schema")
    if not isinstance(schema, int) or isinstance(schema, bool) or schema != 1:
        raise ValueError("runtime report lacks runtime_evidence_schema 1")
    if report.get("runtime_evidence_toolkit") != "TKernel":
        raise ValueError("runtime report toolkit must be TKernel")
    case_rows(report.get("runtime_case_results"))


def cli_arguments() -> argparse.Namespace:
    """Parse join inputs; e.g. `cli_arguments()` from the command line."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=pathlib.Path)
    parser.add_argument("--root", type=pathlib.Path, default=ROOT)
    parser.add_argument("--occt-source", type=pathlib.Path, required=True)
    parser.add_argument("--output", type=pathlib.Path, required=True)
    return parser.parse_args()


def main() -> None:
    """Run the joiner CLI; e.g. `python tools/join_legacy_runtime_evidence.py report.json`."""
    args = cli_arguments()
    root = args.root.resolve()
    parity = read_object(root / "porting/parity.json")
    report = read_object(args.report)
    result = checked_report(report, root, args.occt_source, parity)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")


if __name__ == "__main__":
    main()
