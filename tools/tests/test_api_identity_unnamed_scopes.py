"""The id parser names operator->, anonymous-namespace members and unnamed enums."""

import unittest

from api_crosswalk import canonical_parameter_type, cpp_indexes, match_api_row
from api_identity import parse_upstream_id, portable_qualified_name, unnamed_scope_end

EXTREMA = "src/ModelingData/TKGeomBase/Extrema/Extrema_GGenExtCC.hxx"
BVH = "src/FoundationClasses/TKMath/BVH/BVH_Constants.hxx"


class OperatorArrowTest(unittest.TestCase):
    def test_arrow_is_an_operator_not_a_template_bracket(self) -> None:
        self.assertEqual(
            parse_upstream_id("NCollection_Handle::operator->() const noexcept"),
            ("NCollection_Handle::operator->", [], "const noexcept"),
        )

    def test_arrow_star_takes_parameters(self) -> None:
        self.assertEqual(parse_upstream_id("X::operator->*(int)"), ("X::operator->*", ["int"], ""))

    def test_other_operators_keep_their_parse(self) -> None:
        self.assertEqual(parse_upstream_id("A::operator>>(int)"), ("A::operator>>", ["int"], ""))
        self.assertEqual(parse_upstream_id("A::operator()(int)"), ("A::operator()", ["int"], ""))


class UnnamedScopeTest(unittest.TestCase):
    def test_anonymous_namespace_member_with_parameters(self) -> None:
        identity = "(anonymous namespace)::Inspector::Shift(const gp_XY&, double)"
        self.assertEqual(
            parse_upstream_id(identity),
            ("(anonymous namespace)::Inspector::Shift", ["const gp_XY&", "double"], ""),
        )

    def test_unnamed_enum_and_union_names_have_no_parameters(self) -> None:
        for name in ("(unnamed enum at src/A/B.hxx:17:1)", "L::(unnamed union)::i", "(anonymous)"):
            self.assertEqual(parse_upstream_id(name), (name, None, ""))

    def test_a_parenthesis_inside_a_segment_is_not_a_scope(self) -> None:
        self.assertIsNone(unnamed_scope_end("f(anonymous namespace)", 1))
        self.assertIsNone(unnamed_scope_end("(anonymous namespace)x", 0))
        self.assertEqual(unnamed_scope_end("(anonymous namespace)::F", 0), 21)

    def test_absolute_location_becomes_source_relative(self) -> None:
        name = f"(unnamed enum at /home/u/OCCT/{BVH}:17:1)"
        self.assertEqual(portable_qualified_name(name, BVH), f"(unnamed enum at {BVH}:17:1)")

    def test_foreign_location_and_named_declarations_are_untouched(self) -> None:
        name = "(unnamed enum at /usr/include/x.h:3:1)"
        self.assertEqual(portable_qualified_name(name, BVH), name)
        self.assertEqual(portable_qualified_name("gp_Pnt", BVH), "gp_Pnt")
        self.assertEqual(portable_qualified_name(name, None), name)


class UnnamedCrosswalkTest(unittest.TestCase):
    def setUp(self) -> None:
        self.enum = {"id": "enum", "qualified_name": f"(unnamed enum at /o/{BVH}:17:1)",
                     "kind": "Enum", "signature": "", "source": BVH, "api_visible": True}
        self.method = {"id": "shift", "qualified_name": "(anonymous namespace)::Inspector::Shift",
                       "kind": "CXXMethod", "signature": "class gp_XY (const class gp_XY &, double)",
                       "parameter_types": ["const class gp_XY &", "double"],
                       "written_parameter_types": ["const gp_XY &", "double"],
                       "source": EXTREMA, "api_visible": True}
        owners = {BVH: {"TKMath"}, EXTREMA: {"TKGeomBase"}}
        self.by_name, _, _ = cpp_indexes([self.enum, self.method], owners)

    def row(self, upstream_id: str, toolkit: str, source: str) -> dict:
        return {"upstream_id": upstream_id, "toolkit": toolkit, "source": source,
                "status": "excluded", "notes": "fixture"}

    def test_portable_unnamed_enum_row_resolves(self) -> None:
        row = self.row(f"(unnamed enum at {BVH}:17:1)", "TKMath", BVH)
        self.assertEqual(match_api_row(row, self.by_name, {}, {BVH})["cpp_ids"], ["enum"])

    def test_anonymous_namespace_method_row_resolves(self) -> None:
        row = self.row("(anonymous namespace)::Inspector::Shift(const gp_XY&,double)", "TKGeomBase", EXTREMA)
        self.assertEqual(match_api_row(row, self.by_name, {}, {EXTREMA})["resolution"], "excluded")


class PointerToArrayParameterTest(unittest.TestCase):
    def test_pointer_to_array_keeps_its_declarator(self) -> None:
        # MathSys::Utils::Solve2x2 takes `const double (*)[2]`.
        self.assertEqual(canonical_parameter_type("const double (*)[2]"), "const double(*)[2]")

    def test_function_pointer_still_needs_a_parser(self) -> None:
        self.assertIsNone(canonical_parameter_type("double (*)(int)"))


if __name__ == "__main__":
    unittest.main()
