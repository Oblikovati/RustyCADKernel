"""Replay artifact-pinned partial binding transactions against current source bodies."""

from __future__ import annotations

from pathlib import Path

from api_binding_replay import BindingReplay, binding_equal, verify_binding_ledger
from api_coverage_report import read_json, read_jsonl
from api_crosswalk import cpp_indexes, header_owners, rust_crates
from api_inventory.inputs import JSONObject, digest, object_rows, text_field


def _verified_artifact(root: Path, receipt: JSONObject, label: str) -> Path:
    artifact = root / text_field(receipt, label + "_path")
    expected = receipt.get(label + "_sha256")
    binding_equal(digest(artifact), expected, label + " artifact")
    return artifact


def replay_binding_transaction(root: Path, receipt_path: Path) -> int:
    """Replay an exact partial receipt; `replay_binding_transaction(root, receipt)`."""
    receipt = read_json(root / receipt_path)
    records = read_jsonl(_verified_artifact(root, receipt, "bindings"))
    inventory = read_json(_verified_artifact(root, receipt, "rust_inventory"))
    native = read_jsonl(root / "porting/api-inventory/occt-declarations.jsonl.gz")
    owners = header_owners(read_json(root / "porting/api-header-manifest.json"))
    by_name, _, _ = cpp_indexes(native, owners)
    replay = BindingReplay(
        by_name, rust_crates(inventory), lambda path: digest(root / path)
    )
    for record in records:
        replay.verify(record)
    ledger = object_rows(read_json(root / "porting/parity.json")["api"], "parity api")
    verify_binding_ledger(records, ledger)
    return len(records)
