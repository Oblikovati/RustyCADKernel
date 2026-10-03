#!/usr/bin/env python3
"""Publish legacy test declarations and discovered-but-unverified test inputs."""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import pathlib
from collections import Counter, defaultdict
from api_inventory.inputs import JSONValue, JSONObject, object_rows, text_field

ROOT = pathlib.Path(__file__).resolve().parents[1]
INVENTORY_DEFAULT = ROOT / "porting/inventory.json"
PARITY_DEFAULT = ROOT / "porting/parity.json"
OUTPUT_DEFAULT = ROOT / "porting/api-inventory"


def checked_json_object(path: pathlib.Path) -> JSONObject:
    value: JSONValue = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise ValueError(f"{path}: expected JSON object, got {type(value).__name__}")
    return value


def string_rows(value: JSONObject, key: str) -> list[str]:
    rows = value.get(key)
    if not isinstance(rows, list) or not all(isinstance(row, str) for row in rows):
        raise ValueError(f"{key}: expected array of strings, got {rows!r}")
    return rows


def object_field(value: JSONObject, key: str) -> JSONObject:
    field = value.get(key)
    if not isinstance(field, dict):
        raise ValueError(f"{key}: expected object, got {field!r}")
    return field


def int_field(value: JSONObject, key: str) -> int:
    field = value.get(key)
    if not isinstance(field, int) or isinstance(field, bool):
        raise ValueError(f"{key}: expected integer, got {field!r}")
    return field


def optional_text_field(value: JSONObject, key: str) -> str:
    field = value.get(key)
    if field is None:
        return ""
    if not isinstance(field, str):
        raise ValueError(f"{key}: expected string or null, got {field!r}")
    return field


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


def collect_declarations(inventory: JSONObject) -> list[JSONObject]:
    declarations: list[JSONObject] = []
    for toolkit in object_rows(inventory.get("toolkits"), "toolkits"):
        toolkit_name = text_field(toolkit, "name")
        candidates = object_rows(
            toolkit.get("gtest_declaration_candidates"), toolkit_name
        )
        for candidate in candidates:
            declarations.append({**candidate, "toolkit": toolkit_name})
    return declarations


def parity_index(parity: JSONObject) -> dict[tuple[str, str, str], list[JSONObject]]:
    index: dict[tuple[str, str, str], list[JSONObject]] = defaultdict(list)
    for row in object_rows(parity.get("tests"), "parity tests"):
        key = identity(
            text_field(row, "toolkit"),
            text_field(row, "source"),
            text_field(row, "upstream_id"),
        )
        index[key].append(row)
    return dict(index)


def declaration_counts(declarations: list[JSONObject]) -> Counter[tuple[str, str, str]]:
    return Counter(
        identity(
            text_field(row, "toolkit"),
            text_field(row, "source"),
            text_field(row, "name"),
        )
        for row in declarations
    )


def declaration_id(row: JSONObject, ordinal: int) -> str:
    value = [
        text_field(row, "toolkit"),
        text_field(row, "source"),
        text_field(row, "name"),
        text_field(row, "macro"),
        ordinal,
    ]
    return hashlib.sha256(json.dumps(value, separators=(",", ":")).encode()).hexdigest()


def gtest_records(
    inventory: JSONObject,
    parity: JSONObject,
) -> tuple[list[JSONObject], list[JSONObject]]:
    declarations = collect_declarations(inventory)
    mappings = parity_index(parity)
    counts = declaration_counts(declarations)
    occurrences: Counter[tuple[str, str, str]] = Counter()
    known = set(counts)
    records: list[JSONObject] = []
    anomalies: list[JSONObject] = []
    for declaration in declarations:
        key = identity(
            text_field(declaration, "toolkit"),
            text_field(declaration, "source"),
            text_field(declaration, "name"),
        )
        occurrences[key] += 1
        matches = mappings.get(key, [])
        state = mapping_state(matches, counts[key])
        records.append(make_gtest_record(declaration, occurrences[key], state, matches))
        if state.startswith("duplicate"):
            anomalies.append(mapping_anomaly(key, state, matches, counts[key]))
    anomalies.extend(unknown_mapping_anomalies(mappings, known))
    return records, anomalies


def mapping_state(matches: list[JSONObject], declaration_count: int) -> str:
    if declaration_count > 1:
        return "duplicate_declaration_identity"
    if len(matches) > 1:
        return "duplicate_parity_mapping"
    return "mapped" if matches else "unmapped"


def make_gtest_record(
    declaration: JSONObject,
    ordinal: int,
    state: str,
    matches: list[JSONObject],
) -> JSONObject:
    row = {"kind": "gtest_declaration", **declaration}
    row["declaration_id"] = declaration_id(declaration, ordinal)
    row["mapping_status"] = state
    row["mapped_rust_test"] = matches[0] if state == "mapped" else None
    row["duplicate_mapping_rows"] = matches if len(matches) > 1 else []
    row["runtime_status"] = "unverified"
    return row


def mapping_anomaly(
    key: tuple[str, str, str],
    state: str,
    matches: list[JSONObject],
    count: int,
) -> JSONObject:
    return {
        "kind": "mapping_anomaly",
        "reason": state,
        "toolkit": key[0],
        "source": key[1],
        "upstream_id": key[2],
        "declaration_count": count,
        "parity_rows": matches,
    }


def unknown_mapping_anomalies(
    mappings: dict[tuple[str, str, str], list[JSONObject]],
    known: set[tuple[str, str, str]],
) -> list[JSONObject]:
    return [
        mapping_anomaly(key, "unknown_parity_mapping", rows, 0)
        for key, rows in sorted(mappings.items())
        if key not in known
    ]


def discovered_records(inventory: JSONObject) -> list[JSONObject]:
    draw = [
        discovered_record("draw_case_candidate", path)
        for path in string_rows(inventory, "draw_case_candidates")
    ]
    regression = [
        discovered_record("regression_file", path)
        for path in string_rows(inventory, "upstream_test_files")
    ]
    return draw + regression


def discovered_record(kind: str, path: str) -> JSONObject:
    return {
        "kind": kind,
        "path": path,
        "toolkit": None,
        "discovery_status": "discovered_unverified",
        "runtime_status": "unverified",
    }


def sort_records(records: list[JSONObject]) -> list[JSONObject]:
    return sorted(records, key=record_sort_key)


def record_sort_key(row: JSONObject) -> tuple[str, str, str, str, str]:
    return (
        text_field(row, "kind"),
        optional_text_field(row, "toolkit"),
        optional_text_field(row, "source") or optional_text_field(row, "path"),
        optional_text_field(row, "name") or optional_text_field(row, "upstream_id"),
        optional_text_field(row, "declaration_id"),
    )


def toolkit_summary(
    inventory: JSONObject,
    gtests: list[JSONObject],
    anomalies: list[JSONObject],
) -> dict[str, dict[str, int]]:
    rows: dict[str, dict[str, int]] = {}
    unknown = Counter(
        text_field(row, "toolkit")
        for row in anomalies
        if text_field(row, "reason") == "unknown_parity_mapping"
    )
    for toolkit in object_rows(inventory.get("toolkits"), "toolkits"):
        name = text_field(toolkit, "name")
        owned = [row for row in gtests if text_field(row, "toolkit") == name]
        rows[name] = {
            "declared": len(owned),
            "mapped": count_state(owned, "mapped"),
            "unmapped": count_state(owned, "unmapped"),
            "duplicate_mappings": count_state(owned, "duplicate_parity_mapping"),
            "duplicate_declarations": count_state(
                owned, "duplicate_declaration_identity"
            ),
            "unknown_parity_mappings": unknown[name],
        }
    return rows


def count_state(rows: list[JSONObject], status: str) -> int:
    return sum(row.get("mapping_status") == status for row in rows)


def report_document(
    inventory: JSONObject,
    parity: JSONObject,
    inventory_sha256: str,
    parity_sha256: str,
) -> tuple[list[JSONObject], JSONObject]:
    gtests, anomalies = gtest_records(inventory, parity)
    records = sort_records(gtests + anomalies + discovered_records(inventory))
    summary = summary_document(
        inventory, inventory_sha256, parity_sha256, gtests, anomalies
    )
    return records, summary


def summary_document(
    inventory: JSONObject,
    inventory_sha256: str,
    parity_sha256: str,
    gtests: list[JSONObject],
    anomalies: list[JSONObject],
) -> JSONObject:
    return {
        "schema_version": 1,
        "occt_revision": inventory.get("revision"),
        "inputs": {
            "inventory_sha256": inventory_sha256,
            "parity_sha256": parity_sha256,
        },
        "gtest": gtest_summary(gtests, anomalies),
        "draw_cases": discovered_summary(inventory, "draw_case_candidates"),
        "regression_files": discovered_summary(inventory, "upstream_test_files"),
        "by_toolkit": toolkit_summary(inventory, gtests, anomalies),
        "runtime_proof": False,
        "mapping_inventory_complete": not anomalies
        and not count_state(gtests, "unmapped"),
    }


def gtest_summary(
    gtests: list[JSONObject],
    anomalies: list[JSONObject],
) -> JSONObject:
    return {
        "declared": len(gtests),
        "mapped": count_state(gtests, "mapped"),
        "unmapped": count_state(gtests, "unmapped"),
        "duplicate_parity_mappings": count_state(gtests, "duplicate_parity_mapping"),
        "duplicate_declaration_identities": count_state(
            gtests, "duplicate_declaration_identity"
        ),
        "unknown_parity_mappings": sum(
            text_field(row, "reason") == "unknown_parity_mapping" for row in anomalies
        ),
        "runtime_status": "unverified",
    }


def discovered_summary(inventory: JSONObject, field: str) -> JSONObject:
    return {
        "discovered": len(string_rows(inventory, field)),
        "runtime_status": "unverified",
    }


def write_report(
    output: pathlib.Path,
    records: list[JSONObject],
    summary: JSONObject,
) -> None:
    output.mkdir(parents=True, exist_ok=True)
    payload = "".join(
        json.dumps(row, sort_keys=True, separators=(",", ":")) + "\n" for row in records
    )
    compressed = gzip.compress(payload.encode(), mtime=0)
    table_path = output / "legacy-tests.jsonl.gz"
    table_path.write_bytes(compressed)
    summary["full_table_sha256"] = hashlib.sha256(compressed).hexdigest()
    summary["full_table_records"] = len(records)
    (output / "legacy-test-summary.json").write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n"
    )


def main() -> None:
    args = arguments()
    inventory = checked_json_object(args.inventory)
    parity = checked_json_object(args.parity)
    records, summary = report_document(
        inventory, parity, file_digest(args.inventory), file_digest(args.parity)
    )
    write_report(args.output, records, summary)
    gtest = object_field(summary, "gtest")
    draw = object_field(summary, "draw_cases")
    regressions = object_field(summary, "regression_files")
    print(
        f"Legacy tests: {int_field(gtest, 'mapped')}/{int_field(gtest, 'declared')} mapped; "
        f"{int_field(draw, 'discovered')} DRAW candidates and "
        f"{int_field(regressions, 'discovered')} regression files discovered; runtime proof false"
    )


if __name__ == "__main__":
    main()
