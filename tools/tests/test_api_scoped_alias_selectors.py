"""Only compiler-captured aliases from a member's exact scope can shorten selectors."""

import unittest

from api_crosswalk import cpp_candidates
from api_inventory.inputs import JSONObject
from api_tool_hashes import IMPLEMENTATION_FILES


def alias_catalog() -> dict[str, list[JSONObject]]:
    member: JSONObject = {
        "id": "method",
        "qualified_name": "Buffer::Init",
        "kind": "CXXMethod",
        "parent_usr": "buffer-owner",
        "api_visible": True,
        "_toolkits": {"TKernel"},
        "written_parameter_types": ["const Buffer::value_type &"],
    }
    alias: JSONObject = {
        "id": "alias",
        "qualified_name": "Buffer::value_type",
        "kind": "TypeAlias",
        "parent_usr": "buffer-owner",
        "stable_identity": True,
    }
    return {"Buffer::Init": [member], "Buffer::value_type": [alias]}


class ScopedAliasSelectorTest(unittest.TestCase):
    def test_alias_rules_are_part_of_crosswalk_provenance(self) -> None:
        self.assertIn(
            "tools/api_scoped_alias_selectors.py", IMPLEMENTATION_FILES["api_crosswalk"]
        )

    def assert_resolution(
        self,
        catalog: dict[str, list[JSONObject]],
        expected: str,
        parameter: str = "const value_type&",
    ) -> None:
        status, matches = cpp_candidates(
            f"Buffer::Init({parameter})", "TKernel", catalog
        )
        self.assertEqual(status, expected)
        self.assertEqual(
            [row["id"] for row in matches], ["method"] if expected == "unique" else []
        )

    def test_alias_in_exact_member_scope_is_resolved(self) -> None:
        self.assert_resolution(alias_catalog(), "unique")

    def test_qualified_spelling_remains_resolved(self) -> None:
        self.assert_resolution(alias_catalog(), "unique", "const Buffer::value_type&")

    def test_absent_alias_declaration_is_not_guessed(self) -> None:
        catalog = alias_catalog()
        del catalog["Buffer::value_type"]
        self.assert_resolution(catalog, "missing")

    def test_foreign_scope_does_not_supply_alias(self) -> None:
        catalog = alias_catalog()
        catalog["Buffer::value_type"][0]["parent_usr"] = "different-owner"
        self.assert_resolution(catalog, "missing")

    def test_foreign_qualified_parameter_is_not_shortened(self) -> None:
        self.assert_resolution(alias_catalog(), "missing", "const Other::value_type&")

    def test_reference_and_const_qualifiers_are_preserved(self) -> None:
        for parameter in ["value_type&", "const value_type&&", "const value_type*"]:
            with self.subTest(parameter=parameter):
                self.assert_resolution(alias_catalog(), "missing", parameter)

    def test_conflicted_or_unstable_alias_cannot_supply_a_match(self) -> None:
        for field, value in [("inventory_conflict", True), ("stable_identity", False)]:
            catalog = alias_catalog()
            catalog["Buffer::value_type"][0][field] = value
            self.assert_resolution(catalog, "missing")

    def test_duplicate_alias_obligations_remain_unresolved(self) -> None:
        catalog = alias_catalog()
        catalog["Buffer::value_type"] *= 2
        self.assert_resolution(catalog, "missing")

    def test_function_template_scope_is_not_inferred(self) -> None:
        catalog = alias_catalog()
        catalog["Buffer::Init"][0]["kind"] = "FunctionTemplate"
        self.assert_resolution(catalog, "missing")

    def test_non_type_member_is_not_used_as_a_type_alias(self) -> None:
        catalog = alias_catalog()
        catalog["Buffer::value_type"][0]["kind"] = "FieldDecl"
        self.assert_resolution(catalog, "missing")

    def test_typedef_alias_and_constructor_scope_are_supported(self) -> None:
        catalog = alias_catalog()
        catalog["Buffer::value_type"][0]["kind"] = "Typedef"
        catalog["Buffer::Init"][0]["kind"] = "CXXConstructor"
        catalog["Buffer::Init"][0]["qualified_name"] = "Buffer::Buffer"
        catalog["Buffer::Buffer"] = catalog.pop("Buffer::Init")
        status, matches = cpp_candidates(
            "Buffer::Buffer(const value_type&)", "TKernel", catalog
        )
        self.assertEqual(
            (status, [row["id"] for row in matches]), ("unique", ["method"])
        )

    def test_member_template_signature_prevents_alias_inference(self) -> None:
        catalog = alias_catalog()
        catalog["Buffer::Init"][0]["template_signature"] = "template<class value_type>"
        self.assert_resolution(catalog, "missing")
