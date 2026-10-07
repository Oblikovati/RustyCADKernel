#!/usr/bin/env python3
"""Replay bounded UniversalIterator bodies plus superseding nullable interface bindings."""

from __future__ import annotations

import gzip
import json
from pathlib import Path

from api_inventory.inputs import JSONObject, object_rows, read_object, text_field
from verify_osd_iterator_interface import NATIVE_FIELDS, compiler_rows
from verify_osd_iterator_interface import replay as replay_interface
from verify_returned_private_reachability import verify_files

ROOT = Path(__file__).resolve().parents[1]
RECEIPT = "porting/audits/osd-universal-iterator-v1.json"
SCOPED_IDS = {
    "75eb795690685cfab9774769a528772b738cfcbe21816be97445b4df6eb88b4a": "",
    "5ebd3fd2daa60776d6bac798de41d86e1b29d2b71187bb8534285746c50fd0db": "::new",
    "16ec06ece3506f3a90b32f4bf88385385556dbc5d9e2d887e77512f93ce8d684": "::from_interface",
    "89695ccfe02f9796b811b2989b8699ce022a7c10bf013ad20bb492a3488e60b7": "::<core::clone::Clone>::clone",
    "3abe488350c4b2a773344c7836502c8e19723f972d631e83f2e55c42a4f6b68e": "::assign",
    "9dfaf29a4f59cdef3e954ca4279a34a877c3f7488b97a1d9c3c469a3351e1f17": "::<core::cmp::PartialEq>::ne",
    "f436fd28fe5651ee18f6523f1bac41b0305f760dc0ce74434f3877f8172d8fb6": "::<core::cmp::PartialEq>::eq",
    "6ddfeff5d93f33f56c9bd093b6c8c6742175cb191c970ebe91b7738790fdbc0c": "::increment",
    "44e85539e781d43715f2cb90ef05c4746ac8d9ed697948b9ddb03e4f92566e1c": "::post_increment",
    "84555456be19e9758b55f42d868f4f3995e4cee4adaab50466ec708d1b93d703": "::value",
    "bb5e88bef2d24b478a422dcfedd816c44d28445db1e81aca6bb948d4cc13c9f4": "::value_mut",
}


def verify_wrapper_binding(
    binding: JSONObject, native: list[JSONObject], rust: list[JSONObject]
) -> None:
    """Reject wrong overload or compiler ownership; e.g. `verify_wrapper_binding(row, cpp, rust)`."""
    expected = binding["native"]
    selected = [row for row in native if row["id"] == expected["id"]]
    if [{key: row[key] for key in NATIVE_FIELDS} for row in selected] != [expected]:
        raise ValueError(f"native {expected['id']}: expected exact compiler fields")
    suffix = SCOPED_IDS.get(expected["id"])
    if (
        suffix is None
        or binding["rust"]["path"]
        != "rustycad_tkernel::osd::ParallelUniversalIterator" + suffix
    ):
        raise ValueError(
            f"native {expected['id']}: expected its exact scoped Rust overload"
        )
    candidates = [row for row in rust if row["path"] == binding["rust"]["path"]]
    if candidates != [binding["rust"]]:
        raise ValueError(
            f"Rust {binding['rust']['path']}: expected one exact compiler declaration"
        )
    verify_wrapper_shape(binding, bool(suffix))


def verify_wrapper_shape(binding: JSONObject, callable_member: bool) -> None:
    """Distinguish ten concrete bodies from the record; e.g. `verify_wrapper_shape(row, True)`."""
    rust = binding["rust"]
    expected_kind = "function" if callable_member else "struct"
    if rust["kind"] != expected_kind or binding["status"] != "partial":
        raise ValueError(f"Rust {rust['kind']}: expected partial {expected_kind}")
    if callable_member and rust["signature"].get("has_body") is not True:
        raise ValueError(f"Rust {rust['path']}: expected a concrete compiler body")
    if binding["native"]["api_visible"] is not True:
        raise ValueError(
            f"native {binding['native']['id']}: expected preserved public/protected visibility"
        )


def wrapper_native_rows(root: Path) -> list[JSONObject]:
    """Read only this exact native class; e.g. `wrapper_native_rows(ROOT)`."""
    with gzip.open(
        root / "porting/api-inventory/occt-declarations.jsonl.gz", "rt"
    ) as stream:
        return [row for row in map(json.loads, stream) if row["id"] in SCOPED_IDS]


def replay(root: Path, receipt_path: Path) -> JSONObject:
    """Verify new bodies and the nullable interface without changing legacy ledger credit."""
    receipt = read_object(receipt_path)
    verify_files(root, receipt)
    bindings = object_rows(receipt["bindings"], "bindings")
    if {row["native"]["id"] for row in bindings} != set(SCOPED_IDS) or len(
        bindings
    ) != 11:
        raise ValueError(
            f"bindings {len(bindings)}: expected the eleven exact scoped native IDs"
        )
    native = wrapper_native_rows(root)
    rust = compiler_rows(root, receipt)
    for binding in bindings:
        verify_wrapper_binding(binding, native, rust)
    interface = replay_interface(
        root, root / text_field(receipt, "current_interface_receipt")
    )
    return {
        "wrapper_bodies": 10,
        "wrapper_records": 1,
        "interface": interface,
        "api_denominator_complete": False,
        "implementation_credit": False,
    }


if __name__ == "__main__":
    print(json.dumps(replay(ROOT, ROOT / RECEIPT), sort_keys=True))
