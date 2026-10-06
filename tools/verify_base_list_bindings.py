"""Replay BaseList's recorded declarations, compiler bodies, and partial ledger rows."""

from __future__ import annotations

import json
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


def replay_base_list_bindings(root: Path) -> int:
    """Replay the pinned partial transaction; `replay_base_list_bindings(repo_root)`."""
    receipt = read_json(root / "porting/audits/base-list-ledger-bindings.json")
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


if __name__ == "__main__":
    count = replay_base_list_bindings(Path(__file__).resolve().parents[1])
    print(
        json.dumps({"verified_partial_bindings": count, "implemented_credit_added": 0})
    )
