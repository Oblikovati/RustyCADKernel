#!/usr/bin/env python3
"""Replay the bounded private-return-type capture without promoting global coverage."""

from __future__ import annotations

import argparse
import gzip
import json
from pathlib import Path

from api_inventory.inputs import (
    JSONObject,
    digest,
    object_rows,
    read_object,
    text_field,
)
from api_inventory.merge import InventoryRecords, consume_event, verify_capture_summary

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_RECEIPT = "porting/audits/returned-private-type-extractor-v1.json"


def read_capture(path: Path, source: Path, header: str) -> InventoryRecords:
    """Replay a compressed compiler stream; e.g. `read_capture(path, source, header)`."""
    records = InventoryRecords()
    with gzip.open(path, "rt") as stream:
        for line in stream:
            event: JSONObject = json.loads(line)
            consume_event(records, event, header, source)
    return records


def verify_files(root: Path, receipt: JSONObject) -> None:
    """Reject changed source or evidence bytes; e.g. `verify_files(root, receipt)`."""
    for row in object_rows(receipt["attested_files"], "attested files"):
        path = root / text_field(row, "path")
        if digest(path) != row.get("sha256"):
            raise ValueError(f"{path}: expected SHA256 {row.get('sha256')}")


def verify_capture_pair(
    baseline: InventoryRecords, current: InventoryRecords, receipt: JSONObject
) -> None:
    """Require unchanged primary declarations and the exact returned Block/member identities."""
    if any(
        row.get("severity") in {"error", "fatal"}
        for capture in (baseline, current)
        for row in capture.diagnostics
    ):
        raise ValueError("compiler diagnostics: expected no error or fatal diagnostics")
    if baseline.declarations != current.declarations:
        raise ValueError(
            "primary declarations: expected exact baseline/current equality"
        )
    verify_capture_summary(current, 0, 0, text_field(receipt, "header"))
    if current.unsupported:
        raise ValueError(
            f"current capture: expected no merge gaps, got {current.unsupported!r}"
        )
    edge = producer_edge(current, receipt)
    verify_reached_ids(edge, current, receipt)


def producer_edge(current: InventoryRecords, receipt: JSONObject) -> JSONObject:
    """Require one exact exposing declaration; e.g. `producer_edge(current, receipt)`."""
    edges = [
        row
        for row in current.returned_type_reachability
        if row["producer_declaration"]["id"] == receipt["producer_id"]
    ]
    if len(edges) != 1:
        raise ValueError(f"producer: expected one exact edge, got {len(edges)}")
    return edges[0]


def verify_reached_ids(
    edge: JSONObject, current: InventoryRecords, receipt: JSONObject
) -> None:
    """Reject missing member IDs or private declarations promoted into primary coverage."""
    expected = {
        text_field(row, "id")
        for row in object_rows(receipt["reached_declarations"], "reached declarations")
    }
    actual = {edge["target_declaration"]["id"]} | {
        row["declaration"]["id"] for row in edge["members"]
    }
    if actual != expected or len(actual) != 12:
        raise ValueError(
            f"returned Block surface: expected 12 exact identities {expected!r}, got {actual!r}"
        )
    if any(identifier not in current.declarations for identifier in actual):
        raise ValueError(
            "returned Block surface: expected all IDs in primary declarations"
        )
    if any(current.declarations[identifier]["api_visible"] for identifier in actual):
        raise ValueError(
            "returned Block surface: expected primary API visibility unchanged and false"
        )


def replay(root: Path, receipt_path: Path) -> JSONObject:
    """Check source hashes and bounded native records; e.g. `replay(root, receipt_path)`."""
    receipt = read_object(receipt_path)
    verify_files(root, receipt)
    source = Path(text_field(receipt, "capture_source_root"))
    header = text_field(receipt, "header")
    baseline = read_capture(
        root / text_field(receipt, "baseline_capture"), source, header
    )
    current = read_capture(
        root / text_field(receipt, "current_capture"), source, header
    )
    verify_capture_pair(baseline, current, receipt)
    return {
        "primary_declarations": len(current.declarations),
        "api_candidates": sum(
            row["api_visible"] is True for row in current.declarations.values()
        ),
        "returned_type_edges": len(current.returned_type_reachability),
        "api_denominator_complete": False,
        "coverage_credit": False,
    }


def main() -> None:
    """Run the bounded receipt replay, optionally selecting a historical receipt."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--receipt", type=Path, default=ROOT / DEFAULT_RECEIPT)
    arguments = parser.parse_args()
    print(json.dumps(replay(ROOT, arguments.receipt), sort_keys=True))


if __name__ == "__main__":
    main()
