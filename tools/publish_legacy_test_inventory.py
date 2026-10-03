#!/usr/bin/env python3
"""Publish legacy test declarations and discovered-but-unverified test inputs."""
from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import pathlib
from collections import Counter, defaultdict
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
INVENTORY_DEFAULT = ROOT / "porting/inventory.json"
PARITY_DEFAULT = ROOT / "porting/parity.json"
OUTPUT_DEFAULT = ROOT / "porting/api-inventory"


def arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inventory", type=pathlib.Path, default=INVENTORY_DEFAULT)
    parser.add_argument("--parity", type=pathlib.Path, default=PARITY_DEFAULT)
    parser.add_argument("--output", type=pathlib.Path, default=OUTPUT_DEFAULT)
    return parser.parse_args()


def file_digest(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def identity(toolkit: str, source: str, name: str) -> tuple[str, str, str]:
    return toolkit, source, name


def collect_declarations(inventory: dict[str, Any]) -> list[dict[str, Any]]:
    declarations = []
    for toolkit in inventory.get("toolkits", []):
        toolkit_name = toolkit["name"]
        for candidate in toolkit.get("gtest_declaration_candidates", []):
            declarations.append({**candidate, "toolkit": toolkit_name})
    return declarations


def parity_index(parity: dict[str, Any]) -> dict[tuple[str, str, str], list[dict[str, Any]]]:
    index: dict[tuple[str, str, str], list[dict[str, Any]]] = defaultdict(list)
    for row in parity.get("tests", []):
        key = identity(row.get("toolkit", ""), row.get("source", ""), row.get("upstream_id", ""))
        index[key].append(row)
    return dict(index)


def declaration_counts(declarations: list[dict[str, Any]]) -> Counter[tuple[str, str, str]]:
    return Counter(identity(row["toolkit"], row["source"], row["name"]) for row in declarations)


def declaration_id(row: dict[str, Any], ordinal: int) -> str:
    value = [row["toolkit"], row["source"], row["name"], row["macro"], ordinal]
    return hashlib.sha256(json.dumps(value, separators=(",", ":")).encode()).hexdigest()


def gtest_records(
    inventory: dict[str, Any], parity: dict[str, Any],
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    declarations = collect_declarations(inventory)
    mappings = parity_index(parity)
    counts = declaration_counts(declarations)
    occurrences: Counter[tuple[str, str, str]] = Counter()
    known = set(counts)
    records: list[dict[str, Any]] = []
    anomalies: list[dict[str, Any]] = []
    for declaration in declarations:
        key = identity(declaration["toolkit"], declaration["source"], declaration["name"])
        occurrences[key] += 1
        matches = mappings.get(key, [])
        state = mapping_state(matches, counts[key])
        records.append(make_gtest_record(declaration, occurrences[key], state, matches))
        if state.startswith("duplicate"):
            anomalies.append(mapping_anomaly(key, state, matches, counts[key]))
    anomalies.extend(unknown_mapping_anomalies(mappings, known))
    return records, anomalies


def mapping_state(matches: list[dict[str, Any]], declaration_count: int) -> str:
    if declaration_count > 1:
        return "duplicate_declaration_identity"
    if len(matches) > 1:
        return "duplicate_parity_mapping"
    return "mapped" if matches else "unmapped"


def make_gtest_record(
    declaration: dict[str, Any], ordinal: int, state: str,
    matches: list[dict[str, Any]],
) -> dict[str, Any]:
    row = {"kind": "gtest_declaration", **declaration}
    row["declaration_id"] = declaration_id(declaration, ordinal)
    row["mapping_status"] = state
    row["mapped_rust_test"] = matches[0] if state == "mapped" else None
    row["duplicate_mapping_rows"] = matches if len(matches) > 1 else []
    row["runtime_status"] = "unverified"
    return row


def mapping_anomaly(
    key: tuple[str, str, str], state: str, matches: list[dict[str, Any]], count: int,
) -> dict[str, Any]:
    return {"kind": "mapping_anomaly", "reason": state,
            "toolkit": key[0], "source": key[1], "upstream_id": key[2],
            "declaration_count": count, "parity_rows": matches}


def unknown_mapping_anomalies(
    mappings: dict[tuple[str, str, str], list[dict[str, Any]]],
    known: set[tuple[str, str, str]],
) -> list[dict[str, Any]]:
    return [mapping_anomaly(key, "unknown_parity_mapping", rows, 0)
            for key, rows in sorted(mappings.items()) if key not in known]


def discovered_records(inventory: dict[str, Any]) -> list[dict[str, Any]]:
    draw = [discovered_record("draw_case_candidate", path) for path in inventory.get("draw_case_candidates", [])]
    regression = [discovered_record("regression_file", path) for path in inventory.get("upstream_test_files", [])]
    return draw + regression


def discovered_record(kind: str, path: str) -> dict[str, Any]:
    return {"kind": kind, "path": path, "toolkit": None,
            "discovery_status": "discovered_unverified", "runtime_status": "unverified"}


def sort_records(records: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return sorted(records, key=lambda row: (row["kind"], row.get("toolkit") or "",
                                            row.get("source", row.get("path", "")),
                                            row.get("name", row.get("upstream_id", "")),
                                            row.get("declaration_id", "")))


def toolkit_summary(
    inventory: dict[str, Any], gtests: list[dict[str, Any]], anomalies: list[dict[str, Any]],
) -> dict[str, dict[str, int]]:
    rows: dict[str, dict[str, int]] = {}
    unknown = Counter(row["toolkit"] for row in anomalies if row["reason"] == "unknown_parity_mapping")
    for toolkit in inventory.get("toolkits", []):
        name = toolkit["name"]
        owned = [row for row in gtests if row["toolkit"] == name]
        rows[name] = {"declared": len(owned), "mapped": count_state(owned, "mapped"),
                      "unmapped": count_state(owned, "unmapped"),
                      "duplicate_mappings": count_state(owned, "duplicate_parity_mapping"),
                      "duplicate_declarations": count_state(owned, "duplicate_declaration_identity"),
                      "unknown_parity_mappings": unknown[name]}
    return rows


def count_state(rows: list[dict[str, Any]], status: str) -> int:
    return sum(row["mapping_status"] == status for row in rows)


def report_document(
    inventory: dict[str, Any], parity: dict[str, Any],
    inventory_sha256: str, parity_sha256: str,
) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    gtests, anomalies = gtest_records(inventory, parity)
    records = sort_records(gtests + anomalies + discovered_records(inventory))
    summary = summary_document(inventory, inventory_sha256, parity_sha256,
                               gtests, anomalies)
    return records, summary


def summary_document(
    inventory: dict[str, Any], inventory_sha256: str, parity_sha256: str,
    gtests: list[dict[str, Any]], anomalies: list[dict[str, Any]],
) -> dict[str, Any]:
    return {
        "schema_version": 1,
        "occt_revision": inventory.get("revision"),
        "inputs": {"inventory_sha256": inventory_sha256, "parity_sha256": parity_sha256},
        "gtest": gtest_summary(gtests, anomalies),
        "draw_cases": discovered_summary(inventory, "draw_case_candidates"),
        "regression_files": discovered_summary(inventory, "upstream_test_files"),
        "by_toolkit": toolkit_summary(inventory, gtests, anomalies),
        "runtime_proof": False,
        "mapping_inventory_complete": not anomalies and not count_state(gtests, "unmapped"),
    }


def gtest_summary(
    gtests: list[dict[str, Any]], anomalies: list[dict[str, Any]],
) -> dict[str, Any]:
    return {"declared": len(gtests), "mapped": count_state(gtests, "mapped"),
            "unmapped": count_state(gtests, "unmapped"),
            "duplicate_parity_mappings": count_state(gtests, "duplicate_parity_mapping"),
            "duplicate_declaration_identities": count_state(gtests, "duplicate_declaration_identity"),
            "unknown_parity_mappings": sum(a["reason"] == "unknown_parity_mapping" for a in anomalies),
            "runtime_status": "unverified"}


def discovered_summary(inventory: dict[str, Any], field: str) -> dict[str, Any]:
    return {"discovered": len(inventory.get(field, [])), "runtime_status": "unverified"}


def write_report(
    output: pathlib.Path, records: list[dict[str, Any]], summary: dict[str, Any],
) -> None:
    output.mkdir(parents=True, exist_ok=True)
    payload = "".join(json.dumps(row, sort_keys=True, separators=(",", ":")) + "\n" for row in records)
    compressed = gzip.compress(payload.encode(), mtime=0)
    table_path = output / "legacy-tests.jsonl.gz"
    table_path.write_bytes(compressed)
    summary["full_table_sha256"] = hashlib.sha256(compressed).hexdigest()
    summary["full_table_records"] = len(records)
    (output / "legacy-test-summary.json").write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")


def main() -> None:
    args = arguments()
    inventory = json.loads(args.inventory.read_text())
    parity = json.loads(args.parity.read_text())
    records, summary = report_document(inventory, parity, file_digest(args.inventory), file_digest(args.parity))
    write_report(args.output, records, summary)
    print(f"Legacy tests: {summary['gtest']['mapped']}/{summary['gtest']['declared']} mapped; "
          f"{summary['draw_cases']['discovered']} DRAW candidates and "
          f"{summary['regression_files']['discovered']} regression files discovered; runtime proof false")


if __name__ == "__main__":
    main()
