"""Reject stale typed proxies and unsupported full-parity credit for sparse bases."""

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
from verify_sparse_base_bindings import replay_sparse_base_bindings

ROOT = Path(__file__).resolve().parents[2]


class SparseBaseBindingTest(unittest.TestCase):
    records: list[JSONObject]
    matrix: list[JSONObject]
    native_index: dict[str, list[JSONObject]]
    rust_index: RustCrateIndex

    @classmethod
    def setUpClass(cls: type[SparseBaseBindingTest]) -> None:
        cls.records = read_jsonl(
            ROOT / "porting/audits/sparse-base-ledger-records.jsonl"
        )
        cls.matrix = read_jsonl(
            ROOT / "porting/audits/sparse-base-method-corrections.jsonl"
        )
        native = [binding_object(row, "native") for row in cls.matrix]
        owners = {text_field(native[0], "source"): {"TKernel"}}
        cls.native_index, _, _ = cpp_indexes(native, owners)
        inventory = read_json(
            ROOT / "porting/runtime-evidence/sparse-base-normalized.json.gz"
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

    def test_exact_eight_records_and_current_sources_replay(self) -> None:
        for record in self.records:
            self.replay.verify(record)
        self.assertEqual(len(self.records), 8)
        self.assertEqual(replay_sparse_base_bindings(ROOT), 8)

    def test_stale_typed_size_cannot_substitute_for_erased_size(self) -> None:
        record = copy.deepcopy(self.records[0])
        proposed = binding_object(record, "proposed_row")
        target = "rustycad_tkernel::ncollection::SparseArray::len"
        proposed["rust_item"], proposed["rust_items"] = target, [target]
        with self.assertRaisesRegex(ValueError, "Rust path=.*expected"):
            self.replay.verify(record)

    def test_false_implemented_credit_is_rejected(self) -> None:
        record = copy.deepcopy(self.records[0])
        binding_object(record, "proposed_row")["status"] = "implemented"
        with self.assertRaisesRegex(ValueError, "ledger status=.*expected"):
            self.replay.verify(record)

    def test_wrong_native_method_id_is_rejected(self) -> None:
        record = copy.deepcopy(self.records[0])
        record["cpp_id"] = self.records[1]["cpp_id"]
        with self.assertRaisesRegex(ValueError, "native ID=.*expected"):
            self.replay.verify(record)

    def test_self_adapter_cannot_substitute_for_distinct_exchange(self) -> None:
        record = copy.deepcopy(self.records[-1])
        proposed = binding_object(record, "proposed_row")
        target = "rustycad_tkernel::ncollection::SparseArrayBase::exchange_self"
        proposed["rust_item"], proposed["rust_items"] = target, [target]
        with self.assertRaisesRegex(ValueError, "Rust path=.*expected"):
            self.replay.verify(record)

    def test_stale_compiler_source_hash_is_rejected(self) -> None:
        record = copy.deepcopy(self.records[0])
        binding_object(record, "rust_body")["source_sha256"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "Rust source_sha256=.*expected"):
            self.replay.verify(record)

    def test_three_overclaims_and_four_obsolete_exclusions_are_corrected(self) -> None:
        old = [
            row["previous_ledger_row"]
            for row in self.matrix
            if row["previous_ledger_row"]
        ]
        self.assertEqual(sum(row["status"] == "implemented" for row in old), 3)
        self.assertEqual(sum(row["status"] == "excluded" for row in old), 4)
        self.assertTrue(
            all(row["proposed_row"]["status"] == "partial" for row in self.matrix)
        )


if __name__ == "__main__":
    unittest.main()
