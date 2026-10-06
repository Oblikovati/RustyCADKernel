"""Keep historical sparse evidence immutable and require current exact method bodies."""

from __future__ import annotations

import copy
import unittest
from pathlib import Path

from api_binding_replay import BindingReplay, binding_object
from api_coverage_report import read_json, read_jsonl
from api_crosswalk import cpp_indexes, rust_crates
from api_inventory.inputs import JSONObject, digest, text_field
from api_rust_kind import RustCrateIndex
from test_api_binding_replay import PinnedSourceHashes

ROOT = Path(__file__).resolve().parents[2]


class SparseRemainingBindingTest(unittest.TestCase):
    records: list[JSONObject]
    matrix: list[JSONObject]
    native_index: dict[str, list[JSONObject]]
    rust_index: RustCrateIndex

    @classmethod
    def setUpClass(cls: type[SparseRemainingBindingTest]) -> None:
        cls.records = read_jsonl(
            ROOT / "porting/audits/sparse-base-remaining-bindings.jsonl"
        )
        cls.matrix = read_jsonl(
            ROOT / "porting/audits/sparse-base-remaining-matrix.jsonl"
        )
        native = [binding_object(row, "native") for row in cls.matrix]
        owners = {text_field(native[0], "source"): {"TKernel"}}
        cls.native_index, _, _ = cpp_indexes(native, owners)
        inventory = read_json(
            ROOT / "porting/runtime-evidence/sparse-base-remaining-normalized.json.gz"
        )
        cls.rust_index = rust_crates(inventory)

    def setUp(self) -> None:
        hashes = {
            text_field(body, "source_file"): text_field(body, "source_sha256")
            for body in (binding_object(row, "rust_body") for row in self.records)
        }
        self.source = PinnedSourceHashes(hashes)
        self.replay = BindingReplay(
            self.native_index, self.rust_index, self.source.digest
        )

    def test_twenty_current_bodies_have_unique_exact_bindings(self) -> None:
        self.assertEqual(len(self.records), 20)
        for record in self.records:
            self.replay.verify(record)

    def test_historical_artifacts_and_eight_ledger_rows_remain_unchanged(self) -> None:
        receipt = read_json(ROOT / "porting/audits/sparse-base-remaining-methods.json")
        history = binding_object(receipt, "supersedes")
        for label in ("receipt", "bindings"):
            self.assertEqual(
                digest(ROOT / text_field(history, label)), history[label + "_sha256"]
            )
        old = read_jsonl(ROOT / text_field(history, "bindings"))
        self.assertEqual(len(old), 8)
        for historical, current in zip(old, self.records, strict=False):
            self.assertEqual(historical["cpp_id"], current["cpp_id"])
            self.assertEqual(historical["proposed_row"], current["proposed_row"])

    def test_old_cursor_source_hash_cannot_replace_refreshed_body(self) -> None:
        old = read_jsonl(ROOT / "porting/audits/sparse-base-ledger-records.jsonl")
        historical = next(row for row in old if row["cpp_id"].startswith("a07362e70e"))
        with self.assertRaisesRegex(
            ValueError, "Rust (declaration_sha256|source_sha256)=.*expected"
        ):
            self.replay.verify(historical)

    def test_checked_value_cannot_substitute_for_native_cached_value(self) -> None:
        record = copy.deepcopy(
            next(row for row in self.records if row["cpp_id"].startswith("876b21b53b"))
        )
        proposed = binding_object(record, "proposed_row")
        target = "rustycad_tkernel::ncollection::SparseArrayBaseIterator::value"
        proposed["rust_item"], proposed["rust_items"] = target, [target]
        with self.assertRaisesRegex(ValueError, "Rust path=.*expected"):
            self.replay.verify(record)

    def test_nonnullable_constructor_cannot_substitute_for_optional_body(self) -> None:
        record = copy.deepcopy(
            next(row for row in self.records if row["cpp_id"].startswith("15a0c040ad"))
        )
        proposed = binding_object(record, "proposed_row")
        target = "rustycad_tkernel::ncollection::SparseArrayBaseIterator::new"
        proposed["rust_item"], proposed["rust_items"] = target, [target]
        with self.assertRaisesRegex(ValueError, "Rust path=.*expected"):
            self.replay.verify(record)

    def test_default_null_argument_and_explicit_destructor_are_recorded(self) -> None:
        constructor = next(
            row["native"]
            for row in self.matrix
            if row["native"]["id"].startswith("15a0c040ad")
        )
        self.assertEqual(constructor["default_arguments"][0]["expression"], "nullptr")
        destructor = next(
            row for row in self.records if row["cpp_id"].startswith("8fd428ee07")
        )
        body = binding_object(destructor, "rust_body")
        self.assertTrue(binding_object(body, "signature")["has_body"])
        self.assertTrue(
            text_field(body, "path").endswith("::<core::ops::drop::Drop>::drop")
        )

    def test_generic_iterator_records_receive_no_automatic_credit(self) -> None:
        generic = [
            row
            for row in self.matrix
            if row["status"] == "generic_iterator_contract_not_reconciled_in_batch"
        ]
        self.assertEqual(len(generic), 26)
        for row in generic:
            self.assertNotIn("proposed_row", row)

    def test_exposed_private_block_gap_is_retained_without_automatic_promotion(
        self,
    ) -> None:
        exposed = [
            row
            for row in self.matrix
            if row["status"]
            == "returned_private_type_reachability_requires_inventory_review"
        ]
        self.assertEqual(len(exposed), 12)
        for row in exposed:
            self.assertFalse(row["native"]["api_visible"])
            self.assertNotIn("proposed_row", row)


if __name__ == "__main__":
    unittest.main()
