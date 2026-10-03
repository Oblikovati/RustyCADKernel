#!/usr/bin/env python3
"""Build a conservative report joining OCCT declarations, parity rows, and Rust paths."""

from __future__ import annotations

import argparse
import gzip
import json
import sys
from collections import Counter, defaultdict
from pathlib import Path
from typing import Iterable
from api_inventory.inputs import JSONValue

from api_crosswalk import (
    candidate_status,
    cpp_candidates,
    cpp_indexes,
    declaration_summary,
    declaration_toolkits,
    header_owners,
    match_api_row,
    observed_headers,
    parameter_match,
    resolve_status,
    row_list,
    rust_candidates,
    rust_crates,
    rust_summary,
    suffix_matches,
)
from api_report_inputs import (
    crosswalk_input_hashes,
    mark_conflicted_declarations,
    verify_conflict_artifact,
)
from api_tool_hashes import implementation_hashes, stage_hashes

ROOT = Path(__file__).resolve().parents[1]
CPP_DEFAULT = ROOT / "porting/api-inventory/occt-declarations.jsonl.gz"
CPP_MANIFEST_DEFAULT = ROOT / "porting/api-inventory/occt-manifest.json"
RUST_DEFAULT = ROOT / "target/api-proof/rust-api-inventory.json"
PARITY_DEFAULT = ROOT / "porting/parity.json"
HEADER_MANIFEST_DEFAULT = ROOT / "porting/api-header-manifest.json"
REPORT_DEFAULT = ROOT / "target/api-proof/api-coverage-report.json"


def arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cpp", type=Path, default=CPP_DEFAULT)
    parser.add_argument("--cpp-manifest", type=Path, default=CPP_MANIFEST_DEFAULT)
    parser.add_argument("--rust", type=Path, default=RUST_DEFAULT)
    parser.add_argument("--parity", type=Path, default=PARITY_DEFAULT)
    parser.add_argument("--headers", type=Path, default=HEADER_MANIFEST_DEFAULT)
    parser.add_argument("--output", type=Path, default=REPORT_DEFAULT)
    return parser.parse_args()


def read_json(path: Path) -> dict[str, JSONValue]:
    opener = gzip.open if path.suffix == ".gz" else open
    with opener(path, "rt", encoding="utf-8") as stream:
        value = json.load(stream)
    if not isinstance(value, dict):
        raise ValueError(f"{path} must contain a JSON object")
    return value


def read_jsonl(path: Path) -> list[dict[str, JSONValue]]:
    opener = gzip.open if path.suffix == ".gz" else open
    rows = []
    with opener(path, "rt", encoding="utf-8") as stream:
        for line_number, line in enumerate(stream, 1):
            if not line.strip():
                continue
            value = json.loads(line)
            if not isinstance(value, dict):
                raise ValueError(f"{path}:{line_number} must be a JSON object")
            rows.append(value)
    return rows


def report_tool_hashes(root: Path, rust_doc: dict[str, JSONValue]) -> dict[str, JSONValue]:
    """Bind a report to current crosswalk code and its Rust inventory normalizer."""
    recorded = rust_doc.get("tool_implementation_sha256")
    normalizer = (
        recorded.get("rust_normalizer")
        if isinstance(recorded, dict) and set(recorded) == {"rust_normalizer"}
        else None
    )
    if normalizer != stage_hashes(root, "rust_normalizer"):
        raise ValueError("Rust inventory normalizer hashes are missing or stale")
    return implementation_hashes(root)


def cxx_public(declaration: dict[str, JSONValue]) -> bool:
    return declaration.get("api_visible") is True


def rust_inventory_gaps(document: dict[str, JSONValue]) -> list[str]:
    gaps = []
    if document.get("cross_language_completeness_claim") is True:
        gaps.append("rust_inventory_claims_cross_language_completeness")
    for crate in document.get("crates", []):
        if not isinstance(crate, dict):
            gaps.append("malformed_rust_crate_record")
            continue
        if crate.get("unresolved_links"):
            gaps.append(f"rust_unresolved_links:{crate.get('crate', 'unknown')}")
        unclassified = crate.get("unclassified_kinds", {})
        if isinstance(unclassified, dict) and count_nonzero(
            unclassified.get("counts", {})
        ):
            gaps.append(f"rust_unclassified_items:{crate.get('crate', 'unknown')}")
        unsupported = crate.get("unsupported", {})
        unsupported_counts = (
            unsupported.get("kind_counts", {}) if isinstance(unsupported, dict) else {}
        )
        if count_nonzero(unsupported_counts):
            gaps.append(f"rust_unsupported_items:{crate.get('crate', 'unknown')}")
    return gaps


def header_inventory_gaps(
    manifest: dict[str, JSONValue], source_headers: dict[str, JSONValue]
) -> list[str]:
    gaps = []
    expected = manifest.get("expected_headers")
    expected_source = len(source_headers.get("headers", [])) + len(
        source_headers.get("generated_headers", [])
    )
    if not isinstance(expected, int) or expected != expected_source:
        gaps.append(f"expected_header_count_mismatch:{expected!r}!={expected_source}")
    if manifest.get("declaration_inventory_complete") is not True:
        gaps.append("cxx_declaration_inventory_incomplete")
    if manifest.get("macro_candidates", 0):
        gaps.append("macro_candidate_crosswalk_pending")
    if manifest.get("configuration_matrix_complete") is not True:
        gaps.append("cxx_configuration_matrix_incomplete")
    statuses = manifest.get("headers_by_status")
    if not isinstance(statuses, dict):
        gaps.append("missing_header_status_counts")
    elif any(
        status not in {"parsed", "included_fragment", "parsed_with_context"}
        and count_nonzero(count)
        for status, count in statuses.items()
    ):
        gaps.append("cxx_headers_not_all_parsed")
    return gaps


def count_nonzero(value: JSONValue) -> bool:
    if isinstance(value, int):
        return value > 0
    if isinstance(value, dict):
        return any(count_nonzero(item) for item in value.values())
    return bool(value)


def row_claims(
    rows: list[dict[str, JSONValue]],
) -> tuple[dict[str, list[dict[str, JSONValue]]], list[dict[str, JSONValue]]]:
    claims: dict[str, list[dict[str, JSONValue]]] = defaultdict(list)
    for row in rows:
        if row["cpp_status"] == "unique" and row["cpp_ids"]:
            claims[row["cpp_ids"][0]].append(row)
    duplicates = [
        {"cpp_id": identity, "upstream_ids": [row["upstream_id"] for row in matches]}
        for identity, matches in claims.items()
        if len(matches) > 1
    ]
    return dict(claims), duplicates


def toolkit_report(
    toolkits: Iterable[str],
    declarations: list[dict[str, JSONValue]],
    rows: list[dict[str, JSONValue]],
    claims: dict[str, list[dict[str, JSONValue]]],
) -> dict[str, dict[str, int]]:
    summary = {}
    for toolkit in sorted(set(toolkits)):
        visible = [
            row
            for row in declarations
            if toolkit in row.get("_toolkits", set()) and cxx_public(row)
        ]
        mapped = [row for row in rows if row.get("toolkit") == toolkit]
        summary[toolkit] = {
            "observed_cpp_api": len({row.get("id") for row in visible}),
            "parity_rows": len(mapped),
            "implemented": count_resolution(mapped, "implemented"),
            "cpp_declarations_with_resolved_implemented_claim": len(
                implemented_declaration_ids(mapped)
            ),
            "partial": count_parity_status(mapped, "partial"),
            "excluded": count_resolution(mapped, "excluded"),
            "upstream_unimplemented": count_parity_status(
                mapped, "upstream_unimplemented"
            ),
            "unresolved": sum(
                row["resolution"]
                not in {"implemented", "excluded", "partial", "upstream_unimplemented"}
                for row in mapped
            ),
            "unclaimed_cpp_api": sum(row.get("id") not in claims for row in visible),
        }
    return summary


def count_resolution(rows: list[dict[str, JSONValue]], resolution: str) -> int:
    return sum(row.get("resolution") == resolution for row in rows)


def count_parity_status(rows: list[dict[str, JSONValue]], status: str) -> int:
    return sum(row.get("parity_status") == status for row in rows)


def report_document(
    cpp: list[dict[str, JSONValue]],
    cpp_manifest: dict[str, JSONValue],
    source_headers: dict[str, JSONValue],
    rust_doc: dict[str, JSONValue],
    parity: dict[str, JSONValue],
) -> dict[str, JSONValue]:
    owners = header_owners(source_headers)
    known_sources = {header.get("path") for header in source_headers.get("headers", [])}
    by_name, duplicate_ids, malformed = cpp_indexes(cpp, owners)
    rust = rust_crates(rust_doc)
    api_rows = row_list(parity, "api")
    matched = [match_api_row(row, by_name, rust, known_sources) for row in api_rows]
    claims, duplicate_claims = row_claims(matched)
    visible, unclaimed = visible_declarations(cpp, claims)
    unresolved = unresolved_rows(matched)
    gaps = proof_gaps(
        cpp_manifest,
        source_headers,
        rust_doc,
        duplicate_ids,
        duplicate_claims,
        malformed,
        unresolved,
        unclaimed,
        matched,
    )
    toolkits = {
        owner["toolkit"]
        for row in source_headers["headers"]
        for owner in row.get("owners", [])
    }
    return report_fields(
        cpp,
        cpp_manifest,
        toolkits,
        api_rows,
        matched,
        claims,
        duplicate_claims,
        gaps,
        visible,
        unresolved,
        unclaimed,
    )


def visible_declarations(
    declarations: list[dict[str, JSONValue]],
    claims: dict[str, list[dict[str, JSONValue]]],
) -> tuple[list[dict[str, JSONValue]], list[dict[str, JSONValue]]]:
    visible = [row for row in declarations if cxx_public(row)]
    unclaimed = [
        declaration_summary(row) for row in visible if row.get("id") not in claims
    ]
    return visible, unclaimed


def unresolved_rows(rows: list[dict[str, JSONValue]]) -> list[dict[str, JSONValue]]:
    resolved = {"implemented", "excluded", "partial", "upstream_unimplemented"}
    return [row for row in rows if row.get("resolution") not in resolved]


def proof_gaps(
    cpp_manifest: dict[str, JSONValue],
    source_headers: dict[str, JSONValue],
    rust: dict[str, JSONValue],
    duplicate_ids: list[str],
    duplicate_claims: list[dict[str, JSONValue]],
    malformed: int,
    unresolved: list[dict[str, JSONValue]],
    unclaimed: list[dict[str, JSONValue]],
    matched: list[dict[str, JSONValue]],
) -> list[str]:
    gaps = header_inventory_gaps(cpp_manifest, source_headers) + rust_inventory_gaps(
        rust
    )
    unsupported = cpp_unsupported_count(cpp_manifest)
    if unsupported:
        gaps.append(f"unsupported_cpp_inventory_records:{unsupported}")
    if duplicate_ids:
        gaps.append("duplicate_cpp_declaration_ids")
    if malformed:
        gaps.append(f"malformed_cpp_rows:{malformed}")
    if duplicate_claims:
        gaps.append("duplicate_parity_claims_for_cpp_declaration")
    if unresolved:
        gaps.append("unresolved_parity_rows")
    if unclaimed:
        gaps.append("unclaimed_cpp_declarations")
    if any(
        row.get("parity_status") in {"partial", "upstream_unimplemented"}
        for row in matched
    ):
        gaps.append("partial_or_upstream_unimplemented_api_rows")
    return sorted(set(gaps))


def cpp_unsupported_count(manifest: dict[str, JSONValue]) -> int:
    records = manifest.get("unsupported_records", 0)
    if isinstance(records, list):
        record_count = len(records)
    else:
        record_count = int(records) if isinstance(records, (int, float)) else 0
    legacy_count = manifest.get("unsupported_declarations", 0)
    legacy_count = int(legacy_count) if isinstance(legacy_count, (int, float)) else 0
    return max(record_count, legacy_count)


def implemented_declaration_ids(rows: list[dict[str, JSONValue]]) -> set[str]:
    """Count declaration identities rather than repeated ledger claims."""
    return {
        identity
        for row in rows
        if row.get("resolution") == "implemented"
        for identity in row.get("cpp_ids", [])
        if isinstance(identity, str)
    }


def report_fields(
    cpp: list[dict[str, JSONValue]],
    cpp_manifest: dict[str, JSONValue],
    toolkits: set[str],
    api_rows: list[dict[str, JSONValue]],
    matched: list[dict[str, JSONValue]],
    claims: dict[str, list[dict[str, JSONValue]]],
    duplicate_claims: list[dict[str, JSONValue]],
    gaps: list[str],
    visible: list[dict[str, JSONValue]],
    unresolved: list[dict[str, JSONValue]],
    unclaimed: list[dict[str, JSONValue]],
) -> dict[str, JSONValue]:
    resolutions = Counter(row["resolution"] for row in matched)
    return {
        "schema_version": 1,
        "observed_cpp_api_declarations": len({row.get("id") for row in visible}),
        "parity_api_rows": len(api_rows),
        "matched_declared_mappings": resolutions["implemented"],
        "cpp_declarations_with_resolved_implemented_claim": len(
            implemented_declaration_ids(matched)
        ),
        "accounted_exclusions": resolutions["excluded"],
        "partial_mappings": count_parity_status(matched, "partial"),
        "upstream_unimplemented": count_parity_status(
            matched, "upstream_unimplemented"
        ),
        "unresolved_row_count": len(unresolved),
        "unclaimed_cpp_count": len(unclaimed),
        "cpp_manifest_header_status": cpp_manifest.get("headers_by_status", {}),
        "configuration_matrix_complete": cpp_manifest.get(
            "configuration_matrix_complete"
        )
        is True,
        "declaration_crosswalk_proof": not gaps,
        "behavioral_equivalence_proven": False,
        "proof_gaps": gaps,
        "toolkits": toolkit_report(toolkits, cpp, matched, claims),
        "crosswalk_rows": matched,
        "unresolved_rows": unresolved,
        "unclaimed_cpp_declarations": unclaimed,
        "excluded_rows": rows_for_resolution(matched, "excluded"),
        "partial_rows": rows_for_resolution(matched, "partial"),
        "upstream_unimplemented_rows": rows_for_resolution(
            matched, "upstream_unimplemented"
        ),
        "duplicate_cpp_claims": duplicate_claims,
    }


def rows_for_resolution(
    rows: list[dict[str, JSONValue]], resolution: str
) -> list[dict[str, JSONValue]]:
    return [row for row in rows if row.get("resolution") == resolution]


def print_summary(report: dict[str, JSONValue]) -> None:
    print(f"Observed C++ API declarations: {report['observed_cpp_api_declarations']}")
    print(f"Matched implemented mappings: {report['matched_declared_mappings']}")
    print(f"Unresolved parity rows: {report['unresolved_row_count']}")
    print(f"Unclaimed C++ declarations: {report['unclaimed_cpp_count']}")
    print(f"Declaration crosswalk proof: {report['declaration_crosswalk_proof']}")
    print(
        "Toolkit  C++ API  Rows  Impl  Partial  Excluded  UpstreamGap  Unresolved  Unclaimed"
    )
    for toolkit, counts in report["toolkits"].items():
        print(
            f"{toolkit}  {counts['observed_cpp_api']}  {counts['parity_rows']}  {counts['implemented']}  "
            f"{counts['partial']}  {counts['excluded']}  {counts['upstream_unimplemented']}  "
            f"{counts['unresolved']}  {counts['unclaimed_cpp_api']}"
        )


def main() -> int:
    args = arguments()
    cpp = read_jsonl(args.cpp)
    cpp_manifest = read_json(args.cpp_manifest)
    conflicts = read_jsonl(
        verify_conflict_artifact(cpp_manifest, args.cpp_manifest.parent)
    )
    mark_conflicted_declarations(cpp, conflicts)
    source_headers = read_json(args.headers)
    rust_doc = read_json(args.rust)
    parity = read_json(args.parity)
    report = report_document(cpp, cpp_manifest, source_headers, rust_doc, parity)
    report["input_sha256"] = crosswalk_input_hashes(
        {
            key: getattr(args, key)
            for key in ("cpp", "cpp_manifest", "headers", "rust", "parity")
        },
        cpp_manifest,
    )
    report["tool_implementation_sha256"] = report_tool_hashes(ROOT, rust_doc)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print_summary(report)
    print(f"Report written to {args.output}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(2)
