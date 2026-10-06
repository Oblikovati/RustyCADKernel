"""Reject stale, ambiguous, broadened, or altered exact mapping evidence."""

from __future__ import annotations

import copy
import unittest
from dataclasses import dataclass
from pathlib import Path

from api_binding_replay import BindingReplay, binding_object, verify_binding_ledger
from api_coverage_report import read_json, read_jsonl
from api_crosswalk import cpp_indexes, rust_crates
from api_inventory.inputs import JSONObject, text_field
from api_rust_kind import RustCrateIndex
from verify_base_list_bindings import replay_base_list_bindings

ROOT = Path(__file__).resolve().parents[2]


@dataclass(frozen=True)
class PinnedSourceHashes:
    """Named source fake keeps proof rejection checks independent of filesystem I/O."""

    hashes: dict[str, str]

    def digest(self, filename: str) -> str:
        """Return the injected hash; `fake.digest('source.rs')`."""
        if filename in self.hashes:
            return self.hashes[filename]
        raise ValueError(
            f"source={filename!r}; expected one of {sorted(self.hashes)!r}"
        )


class ExactBindingReplayTest(unittest.TestCase):
    records: list[JSONObject]
    native_index: dict[str, list[JSONObject]]
    rust_index: RustCrateIndex
    fixture_hashes: dict[str, str]

    @classmethod
    def setUpClass(cls: type["ExactBindingReplayTest"]) -> None:
        cls.records = read_jsonl(ROOT / "porting/audits/base-list-ledger-records.jsonl")
        matrix = read_jsonl(
            ROOT / "porting/audits/base-list-api-correction-matrix.jsonl"
        )
        native = [binding_object(row, "native") for row in matrix]
        owners = {text_field(native[0], "source"): {"TKernel"}}
        cls.native_index, _, _ = cpp_indexes(native, owners)
        inventory = read_json(
            ROOT / "porting/runtime-evidence/base-list-ledger-normalized.json.gz"
        )
        cls.rust_index = rust_crates(inventory)
        cls.fixture_hashes = {
            text_field(body, "source_file"): text_field(body, "source_sha256")
            for body in (binding_object(row, "rust_body") for row in cls.records)
        }

    def setUp(self) -> None:
        self.record = copy.deepcopy(self.records[0])
        self.hashes = PinnedSourceHashes(dict(self.fixture_hashes))
        self.replay = BindingReplay(
            self.native_index, self.rust_index, self.hashes.digest
        )
        self.proposed = binding_object(self.record, "proposed_row")

    def test_all_recorded_partial_bindings_replay(self) -> None:
        self.assertEqual(len(self.records), 63)
        for record in self.records:
            self.replay.verify(record)
        ledger = [binding_object(row, "proposed_row") for row in self.records]
        verify_binding_ledger(self.records, ledger)

    def test_current_repository_ledger_replays(self) -> None:
        verified = replay_base_list_bindings(ROOT)
        self.assertEqual(verified, 63)
        self.assertEqual(len(self.records), verified)

    def test_missing_native_name_is_rejected(self) -> None:
        self.proposed["upstream_id"] = "NCollection_BaseList::NotAnObservedMethod()"
        with self.assertRaisesRegex(
            ValueError, "native resolution=.*expected 'unique'"
        ):
            self.replay.verify(self.record)

    def test_wrong_exact_native_id_is_rejected(self) -> None:
        self.record["cpp_id"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "native ID=.*expected"):
            self.replay.verify(self.record)

    def test_wrong_native_signature_is_rejected(self) -> None:
        self.record["cpp_signature"] = "void ()"
        with self.assertRaisesRegex(ValueError, "native signature=.*expected"):
            self.replay.verify(self.record)

    def test_missing_rust_body_path_is_rejected(self) -> None:
        self.proposed["rust_item"] = "rustycad_tkernel::NoSuchCompilerBody"
        self.proposed["rust_items"] = [self.proposed["rust_item"]]
        with self.assertRaisesRegex(ValueError, "Rust resolution=.*expected 'unique'"):
            self.replay.verify(self.record)

    def test_same_path_const_specializations_are_rejected(self) -> None:
        path = "rustycad_tkernel::ncollection::BaseListBasicIterator::new"
        self.proposed["rust_item"], self.proposed["rust_items"] = path, [path]
        with self.assertRaisesRegex(ValueError, "Rust resolution='ambiguous'"):
            self.replay.verify(self.record)

    def test_implemented_status_cannot_replace_partial_proof(self) -> None:
        self.proposed["status"] = "implemented"
        with self.assertRaisesRegex(
            ValueError, "ledger status='implemented'; expected 'partial'"
        ):
            self.replay.verify(self.record)

    def test_declaration_without_body_is_rejected(self) -> None:
        altered = copy.deepcopy(self.rust_index)
        path = text_field(self.proposed, "rust_item")
        body = altered["rustycad_tkernel"][path][0]
        binding_object(body, "signature")["has_body"] = False
        replay = BindingReplay(self.native_index, altered, self.hashes.digest)
        with self.assertRaisesRegex(ValueError, "Rust body=False; expected True"):
            replay.verify(self.record)

    def test_changed_impl_context_is_rejected(self) -> None:
        binding_object(self.record, "rust_body")["impl_context"] = {
            "for": "wrong specialization"
        }
        with self.assertRaisesRegex(ValueError, "Rust impl_context=.*expected"):
            self.replay.verify(self.record)

    def test_changed_declaration_hash_is_rejected(self) -> None:
        binding_object(self.record, "rust_body")["declaration_sha256"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "Rust declaration_sha256=.*expected"):
            self.replay.verify(self.record)

    def test_changed_source_bytes_are_rejected(self) -> None:
        source = text_field(binding_object(self.record, "rust_body"), "source_file")
        self.hashes.hashes[source] = "0" * 64
        with self.assertRaisesRegex(ValueError, "expected"):
            self.replay.verify(self.record)

    def test_missing_source_hash_is_rejected(self) -> None:
        source = text_field(binding_object(self.record, "rust_body"), "source_file")
        del self.hashes.hashes[source]
        with self.assertRaisesRegex(ValueError, "source=.*expected one of"):
            self.replay.verify(self.record)

    def test_malformed_proposed_row_is_rejected(self) -> None:
        self.record["proposed_row"] = ["not an object"]
        with self.assertRaisesRegex(ValueError, "expected a JSON object"):
            self.replay.verify(self.record)

    def test_missing_ledger_row_is_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "ledger .*expected"):
            verify_binding_ledger([self.record], [])
        self.assertEqual(self.proposed["status"], "partial")

    def test_duplicate_ledger_row_is_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "ledger .*expected"):
            verify_binding_ledger([self.record], [self.proposed, self.proposed])
        self.assertEqual(self.proposed["status"], "partial")


if __name__ == "__main__":
    unittest.main()
