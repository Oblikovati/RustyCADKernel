#!/usr/bin/env python3
"""Build a conservative report joining OCCT declarations, parity rows, and Rust paths."""
from __future__ import annotations

import argparse
import gzip
import json
import sys
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any, Iterable

from api_identity import leaf_name, norm_cpp, parse_upstream_id

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


def read_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError(f"{path} must contain a JSON object")
    return value


def read_jsonl(path: Path) -> list[dict[str, Any]]:
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


def row_list(document: dict[str, Any], field: str) -> list[dict[str, Any]]:
    rows = document.get(field)
    if not isinstance(rows, list) or any(not isinstance(row, dict) for row in rows):
        raise ValueError(f"JSON field {field!r} must be an array of objects")
    return rows


def header_owners(manifest: dict[str, Any]) -> dict[str, set[str]]:
    owners: dict[str, set[str]] = defaultdict(set)
    for header in row_list(manifest, "headers"):
        path = header.get("path")
        if not isinstance(path, str):
            continue
        for owner in header.get("owners", []):
            if isinstance(owner, dict) and isinstance(owner.get("toolkit"), str):
                owners[path].add(owner["toolkit"])
    return dict(owners)


def observed_headers(declaration: dict[str, Any]) -> set[str]:
    main_header = declaration.get("main_header")
    result = {main_header} if isinstance(main_header, str) else set()
    for site in declaration.get("sites", []):
        if not isinstance(site, dict):
            continue
        observed = site.get("observed_in")
        if isinstance(observed, str):
            result.add(observed)
        elif isinstance(observed, list):
            result.update(path for path in observed if isinstance(path, str))
    return result


def declaration_toolkits(
    declaration: dict[str, Any], owners: dict[str, set[str]]
) -> set[str]:
    source = declaration.get("source")
    direct = owners.get(source, set()) if isinstance(source, str) else set()
    if direct:
        return direct
    paths = observed_headers(declaration)
    return set().union(*(owners.get(path, set()) for path in paths)) if paths else set()


def cpp_indexes(
    declarations: list[dict[str, Any]], owners: dict[str, set[str]]
) -> tuple[dict[str, list[dict[str, Any]]], list[str], int]:
    by_name: dict[str, list[dict[str, Any]]] = defaultdict(list)
    by_id = {}
    duplicates = set()
    malformed = 0
    for declaration in declarations:
        identity = declaration.get("id")
        name = declaration.get("qualified_name")
        if not isinstance(identity, str) or not isinstance(name, str):
            malformed += 1
            continue
        declaration["_toolkits"] = declaration_toolkits(declaration, owners)
        by_name[name].append(declaration)
        if identity in by_id:
            duplicates.add(identity)
        by_id[identity] = declaration
    return dict(by_name), sorted(duplicates), malformed


def suffix_matches(identity_suffix: str, declaration: dict[str, Any]) -> bool:
    if not identity_suffix:
        return True
    signature = declaration.get("signature")
    if not isinstance(signature, str):
        return False
    return norm_cpp(signature).endswith(identity_suffix)


def parameter_match(params: list[str], declaration: dict[str, Any]) -> bool:
    expected = [norm_cpp(item) for item in params]
    choices = []
    for field in ("written_parameter_types", "parameter_types"):
        value = declaration.get(field)
        if isinstance(value, list) and all(isinstance(item, str) for item in value):
            choices.append([norm_cpp(item) for item in value])
    return expected in choices


def cpp_candidates(
    identity: str, toolkit: str, by_name: dict[str, list[dict[str, Any]]]
) -> tuple[str, list[dict[str, Any]]]:
    parsed = parse_upstream_id(identity)
    if parsed is None:
        return "invalid_upstream_id", []
    name, params, suffix = parsed
    exact = by_name.get(name, [])
    if exact or "::" in name:
        candidates = exact
    else:
        candidates = [row for key, rows in by_name.items() if leaf_name(key) == name for row in rows]
    candidates = [row for row in candidates if toolkit in row.get("_toolkits", set())]
    if params is not None:
        candidates = [row for row in candidates if parameter_match(params, row)]
    candidates = [row for row in candidates if suffix_matches(suffix, row)]
    return candidate_status(candidates), candidates


def candidate_status(candidates: list[dict[str, Any]]) -> str:
    if not candidates:
        return "missing"
    return "unique" if len(candidates) == 1 else "ambiguous"


def rust_crates(document: dict[str, Any]) -> dict[str, dict[str, list[dict[str, Any]]]]:
    result = {}
    for crate in row_list(document, "crates"):
        crate_name = crate.get("crate")
        if not isinstance(crate_name, str):
            continue
        paths: dict[str, list[dict[str, Any]]] = defaultdict(list)
        for item in crate.get("declarations", []):
            if isinstance(item, dict) and isinstance(item.get("path"), str):
                paths[item["path"]].append(item)
        result[crate_name.replace("-", "_")] = dict(paths)
    return result


def rust_candidates(
    rust_item: Any, crates: dict[str, dict[str, list[dict[str, Any]]]]
) -> tuple[str, list[dict[str, Any]]]:
    if not isinstance(rust_item, str) or not rust_item:
        return "no_rust_target_declared", []
    prefix, separator, path = rust_item.partition("::")
    if not separator or prefix not in crates:
        return "unknown_rust_crate", []
    candidates = crates[prefix].get(f"crate::{path}", [])
    return candidate_status(candidates), candidates


def match_api_row(
    row: dict[str, Any], by_name: dict[str, list[dict[str, Any]]],
    rust: dict[str, dict[str, list[dict[str, Any]]]], known_sources: set[str],
) -> dict[str, Any]:
    cpp_status, cpp = cpp_candidates(row.get("upstream_id", ""), row.get("toolkit", ""), by_name)
    rust_status, rust_rows = rust_candidates(row.get("rust_item"), rust)
    source_status = row.get("status", "missing_status")
    source = row.get("source")
    source_known = isinstance(source, str) and source in known_sources
    resolution = resolve_status(source_status, cpp_status, rust_status, source_known)
    return {
        "upstream_id": row.get("upstream_id"), "toolkit": row.get("toolkit"),
        "parity_status": source_status, "resolution": resolution,
        "cpp_status": cpp_status, "cpp_ids": [item.get("id") for item in cpp],
        "cpp_matches": [declaration_summary(item) for item in cpp],
        "rust_status": rust_status, "rust_paths": [item.get("path") for item in rust_rows],
        "rust_matches": [rust_summary(item) for item in rust_rows],
        "source": row.get("source"),
    }


def resolve_status(parity: str, cpp: str, rust: str, source_known: bool = True) -> str:
    if parity == "excluded" and cpp == "missing":
        return "unmatched_exclusion" if source_known else "stale_exclusion"
    if cpp == "ambiguous":
        return "ambiguous_cpp"
    if cpp != "unique":
        return "missing_cpp" if cpp == "missing" else "invalid_cpp_identity"
    if parity == "upstream_unimplemented":
        return "upstream_unimplemented"
    if parity == "partial":
        return "partial"
    if parity == "excluded" and rust == "no_rust_target_declared":
        return "excluded"
    if rust == "ambiguous":
        return "ambiguous_rust"
    if rust != "unique":
        return "missing_rust_target"
    if parity == "excluded":
        return "excluded"
    if parity == "implemented" and rust == "unique":
        return "implemented"
    return "unhandled_parity_status"


def cxx_public(declaration: dict[str, Any]) -> bool:
    return declaration.get("api_visible") is True


def rust_inventory_gaps(document: dict[str, Any]) -> list[str]:
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
        if isinstance(unclassified, dict) and count_nonzero(unclassified.get("counts", {})):
            gaps.append(f"rust_unclassified_items:{crate.get('crate', 'unknown')}")
        unsupported = crate.get("unsupported", {})
        unsupported_counts = unsupported.get("kind_counts", {}) if isinstance(unsupported, dict) else {}
        if count_nonzero(unsupported_counts):
            gaps.append(f"rust_unsupported_items:{crate.get('crate', 'unknown')}")
    return gaps


def header_inventory_gaps(manifest: dict[str, Any], source_headers: dict[str, Any]) -> list[str]:
    gaps = []
    expected = manifest.get("expected_headers")
    expected_source = len(source_headers.get("headers", [])) + len(source_headers.get("generated_headers", []))
    if not isinstance(expected, int) or expected != expected_source:
        gaps.append(f"expected_header_count_mismatch:{expected!r}!={expected_source}")
    if manifest.get("configuration_matrix_complete") is not True:
        gaps.append("cxx_configuration_matrix_incomplete")
    statuses = manifest.get("headers_by_status")
    if not isinstance(statuses, dict):
        gaps.append("missing_header_status_counts")
    elif any(status != "parsed" and count_nonzero(count) for status, count in statuses.items()):
        gaps.append("cxx_headers_not_all_parsed")
    return gaps


def count_nonzero(value: Any) -> bool:
    if isinstance(value, int):
        return value > 0
    if isinstance(value, dict):
        return any(count_nonzero(item) for item in value.values())
    return bool(value)


def row_claims(rows: list[dict[str, Any]]) -> tuple[dict[str, list[dict[str, Any]]], list[dict[str, Any]]]:
    claims: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for row in rows:
        if row["cpp_status"] == "unique" and row["cpp_ids"]:
            claims[row["cpp_ids"][0]].append(row)
    duplicates = [
        {"cpp_id": identity, "upstream_ids": [row["upstream_id"] for row in matches]}
        for identity, matches in claims.items() if len(matches) > 1
    ]
    return dict(claims), duplicates


def toolkit_report(
    toolkits: Iterable[str], declarations: list[dict[str, Any]], rows: list[dict[str, Any]],
    claims: dict[str, list[dict[str, Any]]],
) -> dict[str, dict[str, int]]:
    summary = {}
    for toolkit in sorted(set(toolkits)):
        visible = [row for row in declarations if toolkit in row.get("_toolkits", set()) and cxx_public(row)]
        mapped = [row for row in rows if row.get("toolkit") == toolkit]
        summary[toolkit] = {
            "observed_cpp_api": len({row.get("id") for row in visible}),
            "parity_rows": len(mapped),
            "implemented": count_resolution(mapped, "implemented"),
            "partial": count_parity_status(mapped, "partial"),
            "excluded": count_resolution(mapped, "excluded"),
            "upstream_unimplemented": count_parity_status(mapped, "upstream_unimplemented"),
            "unresolved": sum(row["resolution"] not in {"implemented", "excluded", "partial", "upstream_unimplemented"} for row in mapped),
            "unclaimed_cpp_api": sum(row.get("id") not in claims for row in visible),
        }
    return summary


def count_resolution(rows: list[dict[str, Any]], resolution: str) -> int:
    return sum(row.get("resolution") == resolution for row in rows)


def count_parity_status(rows: list[dict[str, Any]], status: str) -> int:
    return sum(row.get("parity_status") == status for row in rows)


def report_document(
    cpp: list[dict[str, Any]], cpp_manifest: dict[str, Any], source_headers: dict[str, Any],
    rust_doc: dict[str, Any], parity: dict[str, Any],
) -> dict[str, Any]:
    owners = header_owners(source_headers)
    known_sources = {header.get("path") for header in source_headers.get("headers", [])}
    by_name, duplicate_ids, malformed = cpp_indexes(cpp, owners)
    rust = rust_crates(rust_doc)
    api_rows = row_list(parity, "api")
    matched = [match_api_row(row, by_name, rust, known_sources) for row in api_rows]
    claims, duplicate_claims = row_claims(matched)
    visible, unclaimed = visible_declarations(cpp, claims)
    unresolved = unresolved_rows(matched)
    gaps = proof_gaps(cpp_manifest, source_headers, rust_doc, duplicate_ids,
                      duplicate_claims, malformed, unresolved, unclaimed, matched)
    toolkits = {owner["toolkit"] for row in source_headers["headers"] for owner in row.get("owners", [])}
    return report_fields(cpp, cpp_manifest, toolkits, api_rows, matched, claims,
                         duplicate_claims, gaps, visible, unresolved, unclaimed)


def visible_declarations(
    declarations: list[dict[str, Any]], claims: dict[str, list[dict[str, Any]]]
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    visible = [row for row in declarations if cxx_public(row)]
    unclaimed = [declaration_summary(row) for row in visible if row.get("id") not in claims]
    return visible, unclaimed


def unresolved_rows(rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    resolved = {"implemented", "excluded", "partial", "upstream_unimplemented"}
    return [row for row in rows if row.get("resolution") not in resolved]


def proof_gaps(
    cpp_manifest: dict[str, Any], source_headers: dict[str, Any], rust: dict[str, Any],
    duplicate_ids: list[str], duplicate_claims: list[dict[str, Any]], malformed: int,
    unresolved: list[dict[str, Any]], unclaimed: list[dict[str, Any]],
    matched: list[dict[str, Any]],
) -> list[str]:
    gaps = header_inventory_gaps(cpp_manifest, source_headers) + rust_inventory_gaps(rust)
    unsupported = cpp_unsupported_count(cpp_manifest)
    if unsupported:
        gaps.append(f"unsupported_cpp_declarations:{unsupported}")
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
    if any(row.get("parity_status") in {"partial", "upstream_unimplemented"} for row in matched):
        gaps.append("partial_or_upstream_unimplemented_api_rows")
    return sorted(set(gaps))


def cpp_unsupported_count(manifest: dict[str, Any]) -> int:
    records = manifest.get("unsupported_records", 0)
    if isinstance(records, list):
        record_count = len(records)
    else:
        record_count = int(records) if isinstance(records, (int, float)) else 0
    legacy_count = manifest.get("unsupported_declarations", 0)
    legacy_count = int(legacy_count) if isinstance(legacy_count, (int, float)) else 0
    return max(record_count, legacy_count)


def report_fields(
    cpp: list[dict[str, Any]], cpp_manifest: dict[str, Any], toolkits: set[str],
    api_rows: list[dict[str, Any]], matched: list[dict[str, Any]],
    claims: dict[str, list[dict[str, Any]]], duplicate_claims: list[dict[str, Any]],
    gaps: list[str], visible: list[dict[str, Any]], unresolved: list[dict[str, Any]],
    unclaimed: list[dict[str, Any]],
) -> dict[str, Any]:
    resolutions = Counter(row["resolution"] for row in matched)
    return {
        "schema_version": 1,
        "observed_cpp_api_declarations": len({row.get("id") for row in visible}),
        "parity_api_rows": len(api_rows),
        "matched_declared_mappings": resolutions["implemented"],
        "accounted_exclusions": resolutions["excluded"],
        "partial_mappings": count_parity_status(matched, "partial"),
        "upstream_unimplemented": count_parity_status(matched, "upstream_unimplemented"),
        "unresolved_row_count": len(unresolved), "unclaimed_cpp_count": len(unclaimed),
        "cpp_manifest_header_status": cpp_manifest.get("headers_by_status", {}),
        "configuration_matrix_complete": cpp_manifest.get("configuration_matrix_complete") is True,
        "declaration_crosswalk_proof": not gaps,
        "behavioral_equivalence_proven": False,
        "proof_gaps": gaps, "toolkits": toolkit_report(toolkits, cpp, matched, claims),
        "crosswalk_rows": matched, "unresolved_rows": unresolved,
        "unclaimed_cpp_declarations": unclaimed,
        "excluded_rows": rows_for_resolution(matched, "excluded"),
        "partial_rows": rows_for_resolution(matched, "partial"),
        "upstream_unimplemented_rows": rows_for_resolution(matched, "upstream_unimplemented"),
        "duplicate_cpp_claims": duplicate_claims,
    }


def declaration_summary(row: dict[str, Any]) -> dict[str, Any]:
    return {key: row.get(key) for key in ("id", "usr", "qualified_name", "kind", "signature", "source")}


def rust_summary(row: dict[str, Any]) -> dict[str, Any]:
    return {key: row.get(key) for key in ("path", "kind", "signature", "source_file", "source_status")}


def rows_for_resolution(rows: list[dict[str, Any]], resolution: str) -> list[dict[str, Any]]:
    return [row for row in rows if row.get("resolution") == resolution]


def print_summary(report: dict[str, Any]) -> None:
    print(f"Observed C++ API declarations: {report['observed_cpp_api_declarations']}")
    print(f"Matched implemented mappings: {report['matched_declared_mappings']}")
    print(f"Unresolved parity rows: {report['unresolved_row_count']}")
    print(f"Unclaimed C++ declarations: {report['unclaimed_cpp_count']}")
    print(f"Declaration crosswalk proof: {report['declaration_crosswalk_proof']}")
    print("Toolkit  C++ API  Rows  Impl  Partial  Excluded  UpstreamGap  Unresolved  Unclaimed")
    for toolkit, counts in report["toolkits"].items():
        print(f"{toolkit}  {counts['observed_cpp_api']}  {counts['parity_rows']}  {counts['implemented']}  "
              f"{counts['partial']}  {counts['excluded']}  {counts['upstream_unimplemented']}  "
              f"{counts['unresolved']}  {counts['unclaimed_cpp_api']}")


def main() -> int:
    args = arguments()
    cpp = read_jsonl(args.cpp)
    cpp_manifest = read_json(args.cpp_manifest)
    source_headers = read_json(args.headers)
    rust_doc = read_json(args.rust)
    parity = read_json(args.parity)
    report = report_document(cpp, cpp_manifest, source_headers, rust_doc, parity)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print_summary(report)
    print(f"Report written to {args.output}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(2)
