"""Reject overload collapse, stale compiler declarations and body-free wrapper credit."""

from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

from api_inventory.inputs import JSONObject
from verify_osd_universal_iterator import RECEIPT, SCOPED_IDS, verify_wrapper_binding

ROOT = Path(__file__).resolve().parents[2]


class UniversalIteratorBindingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.bindings = json.loads((ROOT / RECEIPT).read_text())["bindings"]

    def replay_one(self, binding: JSONObject) -> None:
        verify_wrapper_binding(binding, [binding["native"]], [binding["rust"]])

    def test_all_exact_scoped_rows_pass(self) -> None:
        self.assertEqual(len(self.bindings), len(SCOPED_IDS))
        for row in self.bindings:
            self.replay_one(row)

    def test_overload_cannot_be_replaced_by_neighbor(self) -> None:
        binding = copy.deepcopy(self.bindings[1])
        binding["rust"] = self.bindings[2]["rust"]
        with self.assertRaisesRegex(ValueError, "exact scoped Rust overload"):
            self.replay_one(binding)

    def test_duplicate_rust_declaration_cannot_be_selected_arbitrarily(self) -> None:
        binding = self.bindings[1]
        with self.assertRaisesRegex(ValueError, "one exact compiler declaration"):
            verify_wrapper_binding(binding, [binding["native"]], [binding["rust"]] * 2)

    def test_concrete_method_without_body_is_rejected(self) -> None:
        binding = copy.deepcopy(self.bindings[1])
        binding["rust"]["signature"]["has_body"] = False
        with self.assertRaisesRegex(ValueError, "concrete compiler body"):
            self.replay_one(binding)

    def test_impl_context_mismatch_is_rejected(self) -> None:
        binding = copy.deepcopy(self.bindings[3])
        binding["rust"]["impl_context"] = None
        with self.assertRaisesRegex(ValueError, "one exact compiler declaration"):
            verify_wrapper_binding(
                binding, [binding["native"]], [self.bindings[3]["rust"]]
            )

    def test_native_visibility_must_stay_public_protected(self) -> None:
        binding = copy.deepcopy(self.bindings[1])
        binding["native"]["api_visible"] = False
        with self.assertRaisesRegex(ValueError, "preserved public/protected"):
            self.replay_one(binding)

    def test_partial_evidence_cannot_be_promoted(self) -> None:
        binding = copy.deepcopy(self.bindings[1])
        binding["status"] = "implemented"
        with self.assertRaisesRegex(ValueError, "expected partial"):
            self.replay_one(binding)

    def test_exact_native_signature_is_required(self) -> None:
        binding = copy.deepcopy(self.bindings[1])
        binding["native"]["signature"] = "void (wrong overload)"
        with self.assertRaisesRegex(ValueError, "exact compiler fields"):
            verify_wrapper_binding(
                binding, [self.bindings[1]["native"]], [binding["rust"]]
            )


if __name__ == "__main__":
    unittest.main()
