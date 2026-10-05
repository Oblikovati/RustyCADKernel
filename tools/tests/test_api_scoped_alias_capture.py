"""Bind alias-selector regressions to the actual pinned Clang capture."""

import gzip
import hashlib
import json
import unittest
from pathlib import Path

from api_crosswalk import cpp_candidates, cpp_indexes, header_owners
from api_inventory.inputs import JSONObject

ROOT = Path(__file__).resolve().parents[2]


class CapturedScopedAliasTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        inventory = ROOT / "porting/api-inventory/occt-declarations.jsonl.gz"
        with gzip.open(inventory, "rt") as stream:
            declarations = [json.loads(line) for line in stream]
        cls.inventory_hash = hashlib.sha256(inventory.read_bytes()).hexdigest()
        manifest = json.loads((ROOT / "porting/api-header-manifest.json").read_text())
        cls.by_name, _, _ = cpp_indexes(declarations, header_owners(manifest))
        audit = ROOT / "porting/audits/compiler-scoped-alias-selectors.json"
        cls.audit = json.loads(audit.read_text())

    def test_captured_aliases_bind_the_recorded_native_members(self) -> None:
        self.assertEqual(self.inventory_hash, self.audit["native_inventory_sha256"])
        bindings = self.audit["recovered_bindings_against_snapshot"]
        self.assertEqual(len(bindings), 41)
        for binding in bindings:
            with self.subTest(selector=binding["selector"]):
                self.assert_binding(binding)

    def assert_binding(self, binding: JSONObject) -> None:
        status, declarations = cpp_candidates(
            binding["selector"], "TKernel", self.by_name
        )
        self.assertEqual(status, "unique")
        self.assertEqual([item["id"] for item in declarations], [binding["cpp_id"]])
        declaration = declarations[0]
        self.assertEqual(declaration["source"], binding["source"])
        self.assertEqual(declaration["signature"], binding["cpp_signature"])
        owner = declaration["qualified_name"].rsplit("::", 1)[0]
        for name, identity in binding["alias_ids"].items():
            aliases = self.by_name[f"{owner}::{name}"]
            self.assertEqual([item["id"] for item in aliases], [identity])
            self.assertEqual(aliases[0]["parent_usr"], declaration["parent_usr"])
