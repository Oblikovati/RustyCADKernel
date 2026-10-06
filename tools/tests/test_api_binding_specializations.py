"""Explicit policy selection requires every independently pinned compiler body."""

from __future__ import annotations

import copy
import gzip
import json
import unittest

from api_binding_replay import BindingReplay, binding_bodies
from api_crosswalk import cpp_indexes, rust_crates
from api_inventory.inputs import JSONObject
from test_api_binding_replay import PinnedSourceHashes
from test_api_rust_declaration_selectors import (
    FIXTURE,
    CURSOR_KEY,
    explicit_body_selector,
)


class ExplicitBindingBodiesTests(unittest.TestCase):
    def setUp(self) -> None:
        with gzip.open(FIXTURE, "rt") as stream:
            fixture = json.load(stream)
        self.rust = rust_crates(fixture)
        self.bodies = self.rust["rustycad_tkernel"][CURSOR_KEY]
        native = fixture["native"][0]
        self.cpp, _, _ = cpp_indexes([native], {native["source"]: {"TKernel"}})
        self.hashes = PinnedSourceHashes(
            {body["source_file"]: body["source_sha256"] for body in self.bodies}
        )
        self.replay = BindingReplay(self.cpp, self.rust, self.hashes.digest)
        self.record: JSONObject = self.proof_record(native)

    def proof_record(self, native: JSONObject) -> JSONObject:
        proposed = {
            "upstream_id": "NCollection_BaseMap::BasicIterator::Key() const noexcept",
            "toolkit": "TKernel",
            "status": "partial",
            "rust_item": CURSOR_KEY,
            "rust_declaration_selectors": [
                explicit_body_selector(body) for body in self.bodies
            ],
        }
        return {
            "cpp_id": native["id"],
            "cpp_signature": native["signature"],
            "proposed_row": proposed,
            "rust_bodies": copy.deepcopy(self.bodies),
        }

    def test_every_selected_key_policy_body_is_verified(self) -> None:
        self.assertEqual(len(binding_bodies(self.record)), 3)
        self.replay.verify(self.record)
        self.assertEqual(self.record["proposed_row"]["status"], "partial")

    def test_body_list_cannot_silently_omit_a_selected_policy(self) -> None:
        self.record["rust_bodies"].pop()
        with self.assertRaisesRegex(ValueError, "Rust body count"):
            self.replay.verify(self.record)

    def test_compiler_body_order_is_explicit(self) -> None:
        self.record["rust_bodies"].reverse()
        with self.assertRaisesRegex(
            ValueError, "Rust (signature|impl_context|declaration_sha256)"
        ):
            self.replay.verify(self.record)

    def test_changed_last_policy_impl_cannot_hide_behind_first_body(self) -> None:
        self.record["rust_bodies"][-1]["impl_context"] = {"for": "incorrect"}
        with self.assertRaisesRegex(ValueError, "Rust impl_context"):
            self.replay.verify(self.record)

    def test_changed_last_policy_source_is_rejected(self) -> None:
        self.record["rust_bodies"][-1]["source_sha256"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "Rust source_sha256"):
            self.replay.verify(self.record)

    def test_current_source_digest_is_independently_required(self) -> None:
        self.hashes.hashes[self.bodies[-1]["source_file"]] = "0" * 64
        with self.assertRaisesRegex(ValueError, "expected"):
            self.replay.verify(self.record)

    def test_single_body_contract_remains_supported(self) -> None:
        body = self.bodies[0]
        self.record["proposed_row"]["rust_declaration_selectors"] = [
            explicit_body_selector(body)
        ]
        self.record["rust_body"] = self.record.pop("rust_bodies")[0]
        self.replay.verify(self.record)

    def test_conflicting_or_empty_body_shapes_are_rejected(self) -> None:
        for bodies in ([], None, [None], "bad"):
            changed = dict(self.record, rust_bodies=bodies)
            with self.assertRaisesRegex(ValueError, "rust_bodies=.*expected"):
                self.replay.verify(changed)
        self.record["rust_body"] = self.bodies[0]
        with self.assertRaisesRegex(ValueError, "without rust_body"):
            self.replay.verify(self.record)

    def test_specialization_selection_cannot_promote_behavior_status(self) -> None:
        self.record["proposed_row"]["status"] = "implemented"
        with self.assertRaisesRegex(ValueError, "ledger status='implemented'"):
            self.replay.verify(self.record)


if __name__ == "__main__":
    unittest.main()
