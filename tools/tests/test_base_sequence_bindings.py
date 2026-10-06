"""Keep BaseSequence's raw, typed, const, and mutable declaration domains distinct."""

from __future__ import annotations

import copy
import unittest
from pathlib import Path

from api_binding_replay import BindingReplay, binding_object
from api_coverage_report import read_json, read_jsonl
from api_crosswalk import cpp_indexes, rust_crates
from api_inventory.inputs import JSONObject, text_field
from api_rust_kind import RustCrateIndex
from test_api_binding_replay import PinnedSourceHashes
from verify_base_sequence_bindings import replay_base_sequence_bindings

ROOT = Path(__file__).resolve().parents[2]


class BaseSequenceBindingTest(unittest.TestCase):
    records: list[JSONObject]
    matrix: list[JSONObject]
    native_index: dict[str, list[JSONObject]]
    rust_index: RustCrateIndex

    @classmethod
    def setUpClass(cls: type["BaseSequenceBindingTest"]) -> None:
        cls.records = read_jsonl(
            ROOT / "porting/audits/base-sequence-ledger-records.jsonl"
        )
        cls.matrix = read_jsonl(ROOT / "porting/audits/base-sequence-api-matrix.jsonl")
        native = [binding_object(row, "native") for row in cls.matrix]
        owners = {text_field(native[0], "source"): {"TKernel"}}
        cls.native_index, _, _ = cpp_indexes(native, owners)
        inventory = read_json(
            ROOT / "porting/runtime-evidence/base-sequence-normalized.json.gz"
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

    def test_all_exact_partial_records_and_current_ledger_replay(self) -> None:
        for record in self.records:
            self.replay.verify(record)
        self.assertEqual(len(self.records), 71)
        self.assertEqual(replay_base_sequence_bindings(ROOT), 71)

    def test_former_pointer_exclusions_now_have_exact_raw_bodies(self) -> None:
        corrected = [
            row
            for row in self.matrix
            if (row.get("previous_ledger_row") or {}).get("status") == "excluded"
        ]
        self.assertEqual(len(corrected), 2)
        for row in corrected:
            proposed = binding_object(row, "proposed_row")
            self.assertEqual(proposed["status"], "partial")
            self.assertIn(
                "BaseSequenceRawIterator::", text_field(proposed, "rust_item")
            )

    def test_const_and_mutable_accessor_bodies_remain_withheld(self) -> None:
        conditional = [
            row
            for row in self.matrix
            if row["status"] == "conditional_body_requires_impl_selector"
        ]
        self.assertEqual(len(conditional), 3)
        for row in conditional:
            self.assertEqual(len(row["candidate_binding_keys"]), 2)
            self.assertNotIn("proposed_row", row)

    def test_wrong_raw_pointer_method_cannot_substitute_for_previous(self) -> None:
        record = copy.deepcopy(
            next(row for row in self.records if row["cpp_id"].startswith("47258ab0"))
        )
        proposed = binding_object(record, "proposed_row")
        target = "rustycad_tkernel::ncollection::BaseSequenceRawIterator::current_node"
        proposed["rust_item"], proposed["rust_items"] = target, [target]
        with self.assertRaisesRegex(ValueError, "Rust path=.*expected"):
            self.replay.verify(record)

    def test_same_path_node_specializations_cannot_be_selected_arbitrarily(
        self,
    ) -> None:
        record = copy.deepcopy(self.records[0])
        proposed = binding_object(record, "proposed_row")
        target = "rustycad_tkernel::ncollection::BaseSequenceBasicIterator::node"
        proposed["rust_item"], proposed["rust_items"] = target, [target]
        with self.assertRaisesRegex(ValueError, "Rust resolution='ambiguous'"):
            self.replay.verify(record)

    def test_native_conversion_directions_keep_separate_exact_ids(self) -> None:
        conversions = [
            row
            for row in self.records
            if binding_object(row, "proposed_row")["rust_item"]
            == "rustycad_tkernel::ncollection::BaseSequenceBasicIterator::convert"
        ]
        self.assertEqual(len(conversions), 2)
        self.assertEqual(len({row["cpp_id"] for row in conversions}), 2)
        self.assertEqual(len({row["cpp_signature"] for row in conversions}), 2)

    def test_value_remains_readonly_for_the_generic_cursor(self) -> None:
        record = next(
            row for row in self.records if row["cpp_id"].startswith("e5fb702b")
        )
        signature = binding_object(binding_object(record, "rust_body"), "signature")
        output = binding_object(binding_object(signature, "sig"), "output")
        self.assertFalse(binding_object(output, "borrowed_ref")["is_mutable"])


if __name__ == "__main__":
    unittest.main()
