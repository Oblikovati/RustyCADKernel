"""Match a narrow array-reference grammar without erasing dimensions or cv qualifiers."""

import unittest
from api_crosswalk import canonical_parameter_type, cpp_candidates, parameter_match
from api_inventory.inputs import JSONObject


def array_reference_declaration(parameter: str) -> JSONObject:
    """Build an exact fixture, e.g. `array_reference_declaration("const char (&)[N]")`."""
    return {
        "id": "literal-equality",
        "qualified_name": "TCollection_AsciiString::IsEqual",
        "kind": "FunctionTemplate",
        "signature": f"bool ({parameter}) const",
        "parameter_types": [parameter],
        "written_parameter_types": [parameter],
        "api_visible": True,
        "_toolkits": {"TKernel"},
    }


class ArrayReferenceSelectorsTest(unittest.TestCase):
    def test_literal_template_matches_the_compiler_parameter(self) -> None:
        row = array_reference_declaration("const char (&)[N]")
        status, matches = cpp_candidates(
            "TCollection_AsciiString::IsEqual(const char(&)[N]) const",
            "TKernel",
            {"TCollection_AsciiString::IsEqual": [row]},
        )
        self.assertEqual(status, "unique")
        self.assertEqual(matches, [row])

    def test_array_reference_retains_cv_and_dimensions(self) -> None:
        self.assertEqual(
            canonical_parameter_type("const char (&)[N]"), "const char(&)[N]"
        )
        self.assertEqual(canonical_parameter_type("int (&&)[2][3]"), "int(&&)[2][3]")
        self.assertEqual(canonical_parameter_type("ns::Byte (&)[7]"), "ns::Byte(&)[7]")

    def test_different_element_cv_reference_or_bound_cannot_match(self) -> None:
        row = array_reference_declaration("const char (&)[N]")
        for parameter in [
            "char(&)[N]",
            "const char(&&)[N]",
            "const char(&)[M]",
            "const char(&)[7]",
            "const char*",
            "const char(&)[N][2]",
        ]:
            with self.subTest(parameter=parameter):
                self.assertFalse(parameter_match([parameter], row))

    def test_complex_or_incomplete_declarators_remain_unmatched(self) -> None:
        for parameter in [
            "int (*)(int)",
            "int(&)(int)",
            "char(*)[N]",
            "char(&value)[N]",
            "char(&)[]",
            "char(&)[N+1]",
            "char(&)[",
            "std::array<char,2>(&)[N]",
        ]:
            with self.subTest(parameter=parameter):
                self.assertIsNone(canonical_parameter_type(parameter))

    def test_duplicate_array_reference_candidates_remain_ambiguous(self) -> None:
        row = array_reference_declaration("const char (&)[N]")
        status, _ = cpp_candidates(
            "TCollection_AsciiString::IsEqual(const char(&)[N]) const",
            "TKernel",
            {"TCollection_AsciiString::IsEqual": [row, dict(row, id="duplicate")]},
        )
        self.assertEqual(status, "ambiguous")
