"""Fail closed for abstract declarations, inferred private APIs and compiler identity."""

from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

from api_inventory.inputs import JSONObject
from verify_osd_iterator_interface import verify_binding, verify_rust_shape

ROOT = Path(__file__).resolve().parents[2]
# The original non-null contract receipt remains a historical snapshot.
RECEIPT = "porting/audits/osd-universal-interface-bindings-v1.json"


class OsdIteratorInterfaceBindingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.bindings = json.loads((ROOT / RECEIPT).read_text())["bindings"]

    def replay_one(self, binding: JSONObject) -> None:
        verify_binding(binding, [binding["native"]], [binding["rust"]], ROOT)

    def test_current_nullable_contract_keeps_original_box_snapshot(self) -> None:
        historical_path = ROOT / "porting/audits/osd-iterator-interface-v1.json"
        historical = json.loads(historical_path.read_text())["bindings"][4]
        current = self.bindings[4]
        self.assertEqual(historical["native"], current["native"])
        old_output = historical["rust"]["signature"]["sig"]["output"]["resolved_path"]
        new_output = current["rust"]["signature"]["sig"]["output"]["resolved_path"]
        self.assertEqual(old_output["path"], "Box")
        self.assertEqual(new_output["path"], "Option")

    def test_five_scoped_bindings_pass(self) -> None:
        for binding in self.bindings:
            self.replay_one(binding)

    def test_abstract_declaration_does_not_require_an_invented_body(self) -> None:
        binding = next(
            row for row in self.bindings if row["evidence_mode"] == "abstract_dispatch"
        )
        self.assertFalse(binding["rust"]["signature"]["has_body"])
        self.replay_one(binding)

    def test_abstract_body_confusion_rejected(self) -> None:
        binding = copy.deepcopy(self.bindings[2])
        binding["rust"]["signature"]["has_body"] = True
        with self.assertRaisesRegex(ValueError, "shape"):
            self.replay_one(binding)

    def test_primary_private_visibility_cannot_be_promoted(self) -> None:
        binding = copy.deepcopy(self.bindings[0])
        binding["native"]["api_visible"] = True
        with self.assertRaisesRegex(ValueError, "unchanged private"):
            self.replay_one(binding)

    def test_duplicate_rust_path_is_ambiguous(self) -> None:
        binding = self.bindings[0]
        with self.assertRaisesRegex(ValueError, "one exact compiler"):
            verify_binding(binding, [binding["native"]], [binding["rust"]] * 2, ROOT)

    def test_wrong_native_id_cannot_resolve(self) -> None:
        binding = copy.deepcopy(self.bindings[0])
        binding["native"]["id"] = "wrong-id"
        with self.assertRaisesRegex(ValueError, "unchanged private"):
            verify_binding(
                binding, [self.bindings[0]["native"]], [binding["rust"]], ROOT
            )

    def test_stale_source_hash_fails(self) -> None:
        binding = copy.deepcopy(self.bindings[0])
        binding["rust"]["source_sha256"] = "wrong-hash"
        with self.assertRaisesRegex(ValueError, "compiler source hash"):
            verify_rust_shape(binding, binding["rust"], ROOT)

    def test_comparison_extension_cannot_claim_native_clone(self) -> None:
        binding = copy.deepcopy(self.bindings[4])
        binding["rust"]["path"] = (
            "rustycad_tkernel::osd::ParallelIteratorInterface::comparison_value"
        )
        with self.assertRaisesRegex(ValueError, "scoped binding"):
            self.replay_one(binding)

    def test_implementation_credit_cannot_replace_partial(self) -> None:
        binding = copy.deepcopy(self.bindings[0])
        binding["status"] = "implemented"
        with self.assertRaisesRegex(ValueError, "bounded partial"):
            self.replay_one(binding)


if __name__ == "__main__":
    unittest.main()
