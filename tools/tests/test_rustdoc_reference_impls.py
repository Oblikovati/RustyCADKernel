"""Borrowed-Self trait bodies remain distinct from owned implementations."""

from __future__ import annotations

import copy
import unittest
from pathlib import Path

from api_inventory_json import JSONValue, object_value, array_value
from normalize_rustdoc_inventory import make_reachable_inventory
from rustdoc_reference_impls import (
    annotate_reference_trait,
    implementation_member_path,
    reference_self_target,
    valid_reference_lifetime,
    valid_reference_shape,
)
from rustdoc_trait_paths import renamed_trait_member
from tools.tests.test_normalize_rustdoc_inventory import rustdoc_date_period_fixture

ROOT = Path(__file__).resolve().parents[2]


class ReferenceInventoryFixture:
    def __init__(self, mutable: bool = False, lifetime: str | None = None) -> None:
        self.document: dict[str, JSONValue] = rustdoc_date_period_fixture()
        self.implementation = self.attach_implementation()
        self.implementation["for"] = {
            "borrowed_ref": {
                "lifetime": lifetime,
                "is_mutable": mutable,
                "type": {"resolved_path": {"path": "Date", "id": 174, "args": None}},
            }
        }

    def attach_implementation(self) -> dict[str, JSONValue]:
        """Attach the editable fixture impl; e.g. `fixture.attach_implementation()`."""
        index = object_value(self.document["index"], "fixture index")
        node = object_value(index["176"], "fixture impl")
        inner = object_value(node["inner"], "fixture inner")
        implementation = object_value(inner["impl"], "fixture implementation")
        inner["impl"] = implementation
        node["inner"] = inner
        index["176"] = node
        self.document["index"] = index
        return implementation

    def inventory(self) -> dict[str, JSONValue]:
        return make_reachable_inventory(self.document, ROOT, "rustycad_tkernel")

    def reference(self) -> dict[str, JSONValue]:
        receiver = self.implementation["for"]
        assert isinstance(receiver, dict)
        reference = receiver["borrowed_ref"]
        assert isinstance(reference, dict)
        return reference

    def paths(self) -> set[str]:
        rows = array_value(self.inventory()["declarations"], "fixture declarations")
        return {str(object_value(row, "fixture declaration")["path"]) for row in rows}


class ReferenceImplementationTests(unittest.TestCase):
    def test_shared_receiver_keeps_ufcs_and_its_nominal_owner(self) -> None:
        fixture = ReferenceInventoryFixture()
        path = "<&rustycad_tkernel::quantity::Date as core::ops::arith::Add<rustycad_tkernel::quantity::period::Period>>::add"
        self.assertIn(path, fixture.paths())
        self.assertEqual(fixture.inventory()["unresolved_links"], [])
        self.assertFalse(any("Period as core::ops" in row for row in fixture.paths()))

    def test_mutable_receiver_does_not_collapse_into_shared_self(self) -> None:
        paths = ReferenceInventoryFixture(mutable=True).paths()
        self.assertTrue(
            any(
                row.startswith("<&mut rustycad_tkernel::quantity::Date as ")
                for row in paths
            )
        )
        self.assertFalse(
            any(
                row.startswith("<&rustycad_tkernel::quantity::Date as ")
                for row in paths
            )
        )

    def test_explicit_lifetime_survives_the_receiver_path(self) -> None:
        paths = ReferenceInventoryFixture(lifetime="'static").paths()
        self.assertTrue(
            any(
                row.startswith("<&'static rustycad_tkernel::quantity::Date as ")
                for row in paths
            )
        )

    def test_missing_mutability_stays_unresolved(self) -> None:
        fixture = ReferenceInventoryFixture()
        fixture.reference().pop("is_mutable")
        self.assertIsNone(reference_self_target(fixture.implementation))
        self.assertIn(
            "impl:176:unknown-self-type", fixture.inventory()["unresolved_links"]
        )

    def test_unknown_reference_fields_and_mutability_are_rejected(self) -> None:
        reference = ReferenceInventoryFixture().reference()
        reference["extra"] = "unexpected"
        self.assertFalse(valid_reference_shape(reference))
        reference.pop("extra")
        reference["is_mutable"] = "false"
        self.assertFalse(valid_reference_shape(reference))

    def test_lifetimes_require_compiler_identifier_syntax(self) -> None:
        for lifetime in ["'a", "'static", "'_", "'a1"]:
            self.assertTrue(valid_reference_lifetime(lifetime))
        for lifetime in ["", "'", "a", "'a b", "'12"]:
            self.assertFalse(valid_reference_lifetime(lifetime))
        self.assertIsNone(
            reference_self_target(
                ReferenceInventoryFixture(lifetime="'12").implementation
            )
        )

    def test_reference_to_an_unknown_nominal_owner_is_not_attached(self) -> None:
        fixture = ReferenceInventoryFixture()
        fixture.reference()["type"] = {"generic": "T"}
        self.assertIn(
            "impl:176:unknown-self-type", fixture.inventory()["unresolved_links"]
        )

    def test_unreachable_borrowed_owner_never_gains_a_callable_body(self) -> None:
        fixture = ReferenceInventoryFixture()
        index = object_value(fixture.document["index"], "fixture index")
        module = object_value(index["997"], "quantity module")
        module["inner"] = {"module": {"items": [116]}}
        index["997"] = module
        fixture.document["index"] = index
        self.assertIn(
            "impl:176:unreachable-self-owner:174",
            fixture.inventory()["unresolved_links"],
        )
        self.assertFalse(any("<&" in path for path in fixture.paths()))

    def test_owned_and_inherent_member_paths_are_unchanged(self) -> None:
        owned: dict[str, JSONValue] = {"for": {"resolved_path": {"id": 4}}}
        self.assertEqual(
            implementation_member_path("Value", "add", "Add<T>", owned),
            "Value::<Add<T>>::add",
        )
        self.assertEqual(
            implementation_member_path("Value", "new", None, owned), "Value::new"
        )

    def test_reference_trait_alias_renaming_keeps_reference_and_arguments(self) -> None:
        fixture = ReferenceInventoryFixture()
        row: dict[str, JSONValue] = {
            "path": "<&crate::Value as crate::hidden::Operation<f64>>::apply",
            "impl_context": {
                "trait": {"resolved_item_path": "crate::hidden::Operation"}
            },
        }
        row = annotate_reference_trait(
            row, "crate::hidden::Operation<f64>", fixture.implementation
        )
        renamed = renamed_trait_member(row, "crate::PublicOperation")
        self.assertEqual(
            renamed["path"], "<&crate::Value as crate::PublicOperation<f64>>::apply"
        )
        self.assertNotEqual(row["declaration_sha256"], renamed["declaration_sha256"])

    def test_annotation_requires_a_canonical_trait_identity(self) -> None:
        fixture = ReferenceInventoryFixture()
        row: dict[str, JSONValue] = {"path": "some path", "impl_context": {"trait": {}}}
        self.assertEqual(
            annotate_reference_trait(
                copy.deepcopy(row), "Trait", fixture.implementation
            ),
            row,
        )
        self.assertEqual(
            annotate_reference_trait(copy.deepcopy(row), None, fixture.implementation),
            row,
        )

    def test_receiver_context_is_preserved_without_fabricating_source(self) -> None:
        row = self.add_declaration(ReferenceInventoryFixture().inventory(), "<&")
        context = object_value(row["impl_context"], "context")
        self.assertIn("borrowed_ref", object_value(context["for"], "receiver"))
        original = make_reachable_inventory(
            rustdoc_date_period_fixture(), ROOT, "rustycad_tkernel"
        )
        owned = self.add_declaration(original, "rustycad_tkernel::quantity::Date::")
        self.assertEqual(row["source_sha256"], owned["source_sha256"])
        self.assertEqual(row["source_status"], owned["source_status"])
        self.assertIn("reference_impl_trait_canonical", row)

    def add_declaration(
        self, inventory: dict[str, JSONValue], prefix: str
    ) -> dict[str, JSONValue]:
        rows = array_value(inventory["declarations"], "declarations")
        return next(
            object_value(value, "declaration")
            for value in rows
            if str(object_value(value, "declaration")["path"]).startswith(prefix)
            and str(object_value(value, "declaration")["path"]).endswith("::add")
        )


if __name__ == "__main__":
    unittest.main()
