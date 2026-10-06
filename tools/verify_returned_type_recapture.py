#!/usr/bin/env python3
"""Authenticate staged recapture artifacts and replay every unchanged primary identity."""

from __future__ import annotations

import argparse
import gzip
import json
from pathlib import Path

from api_inventory.inputs import JSONObject, read_object, text_field
from api_inventory.merge import declaration_identity
from report_returned_type_inventory import COMPARISON_FIELDS, read_table
from verify_returned_private_reachability import verify_files

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_RECEIPT = ROOT / "porting/audits/returned-type-scope-recapture-v1.json"


def verify_comparison_row(row: JSONObject, declaration: JSONObject) -> None:
    """Require an exact official identity and unchanged semantic projection for one row."""
    identifier = text_field(row, "id")
    if declaration_identity(declaration) != identifier:
        raise ValueError(
            f"declaration {identifier}: expected recomputed exact native identity"
        )
    projection = {field: declaration.get(field) for field in COMPARISON_FIELDS}
    if (
        row.get("state") != "unchanged"
        or row.get("current") != projection
        or row.get("published") != projection
    ):
        raise ValueError(
            f"declaration {identifier}: expected unchanged current and published semantic fields"
        )


def recapture_index(inventory: Path) -> dict[str, JSONObject]:
    """Authenticate and index primary declarations by their exact IDs."""
    return {text_field(row, "id"): row for row in read_table(inventory, "declarations")}


def replay_primary_matrix(inventory: Path, report: Path) -> int:
    """Stream all recorded comparison rows and reject missing or duplicated primary identities."""
    declarations = recapture_index(inventory)
    observed: set[str] = set()
    with gzip.open(report / "primary-comparison.jsonl.gz", "rt") as stream:
        for line in stream:
            row: JSONObject = json.loads(line)
            identifier = text_field(row, "id")
            if identifier not in declarations or identifier in observed:
                raise ValueError(
                    f"declaration {identifier}: expected one known primary identity"
                )
            verify_comparison_row(row, declarations[identifier])
            observed.add(identifier)
    if observed != declarations.keys():
        raise ValueError(
            f"comparison coverage: expected {len(declarations)} IDs, got {len(observed)}"
        )
    return len(observed)


def verify_summary(
    receipt: JSONObject, inventory: JSONObject, report: JSONObject
) -> None:
    """Keep parity credit withheld while requiring the recorded global counts and statuses."""
    expected = receipt.get("global_expected")
    if not isinstance(expected, dict):
        raise TypeError(f"global_expected: expected object, got {expected!r}")
    verify_recapture_counts(expected, inventory)
    if report.get("primary_id_comparison") != {"unchanged": expected["declarations"]}:
        raise ValueError(
            "primary comparison: expected only unchanged historical identities"
        )
    if (
        report.get("coverage_credit") is not False
        or report.get("api_denominator_complete") is not False
    ):
        raise ValueError(
            "reachability credit: expected false credit and denominator completeness"
        )


def verify_recapture_counts(expected: JSONObject, inventory: JSONObject) -> None:
    """Reject changed global counts and header status tallies; e.g. `verify_recapture_counts(expected, manifest)`."""
    for field in [
        "declarations",
        "api_declarations",
        "headers_by_status",
        "unsupported_records",
    ]:
        if inventory.get(field) != expected.get(field):
            raise ValueError(
                f"{field}: expected {expected.get(field)!r}, got {inventory.get(field)!r}"
            )


def replay(root: Path, receipt_path: Path, deep: bool) -> JSONObject:
    """Check all hashes, optionally all semantic fields; e.g. `replay(root, receipt, True)`."""
    receipt = read_object(receipt_path)
    verify_files(root, receipt)
    inventory = root / text_field(receipt, "global_inventory")
    report = root / text_field(receipt, "global_report")
    manifest, summary = (
        read_object(inventory / "occt-manifest.json"),
        read_object(report / "report.json"),
    )
    verify_summary(receipt, manifest, summary)
    identities = replay_primary_matrix(inventory, report) if deep else None
    return {
        "verified_primary_identities": identities,
        "api_candidates": manifest["api_declarations"],
        "distinct_reached_members": summary["distinct_reached_members"],
        "coverage_credit": False,
        "api_denominator_complete": False,
    }


def main() -> None:
    """Replay artifact hashes by default; use --deep for every semantic declaration field."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--receipt", type=Path, default=DEFAULT_RECEIPT)
    parser.add_argument("--deep", action="store_true")
    args = parser.parse_args()
    print(json.dumps(replay(ROOT, args.receipt, args.deep), sort_keys=True))


if __name__ == "__main__":
    main()
