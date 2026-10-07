"""Explicit row pins pick one C++ declaration among same-name declarations; never a guess."""

import unittest

from api_cpp_selectors import CppPin, pinned_candidates, row_cpp_pin
from api_crosswalk import cpp_candidates, match_api_row
from api_inventory.inputs import JSONObject

ITER = "src/ModelingData/TKBRep/BRepGraph/BRepGraph_Iterator.hxx"
NODE = "src/ModelingData/TKBRep/BRepGraph/BRepGraph_NodeId.hxx"
PROPS = "src/ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp_MeshProps.hxx"
BVH = "src/ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_PolyhedronBVH.hxx"


def declaration(identity: str, name: str, kind: str, source: str, **extra: object) -> JSONObject:
    """Visible TK fixture record; e.g. `declaration("a", "N::T", "Enum", PROPS)`."""
    row: JSONObject = {"id": identity, "qualified_name": name, "kind": kind,
                       "signature": "", "source": source, "usr": f"c:@{identity}",
                       "api_visible": True, "_toolkits": {"TK"}}
    row.update(extra)
    return row


def ledger_row(upstream_id: str, source: str, **pin: object) -> JSONObject:
    row: JSONObject = {"upstream_id": upstream_id, "toolkit": "TK", "source": source,
                       "status": "excluded", "notes": "fixture"}
    row.update(pin)
    return row


class SelectorParsingTest(unittest.TestCase):
    def test_unpinned_row_has_no_pin(self) -> None:
        self.assertEqual(row_cpp_pin({"upstream_id": "A"}), (None, None))

    def test_selector_and_id_are_read(self) -> None:
        pin, error = row_cpp_pin({"cpp_selector": {"kind": "Enum"}, "cpp_declaration_id": "x"})
        self.assertIsNone(error)
        self.assertEqual(pin, CppPin("x", {"kind": "Enum"}))

    def test_unknown_or_blank_fields_are_errors_with_the_value(self) -> None:
        for selector in ({"knd": "Enum"}, {"kind": ""}, {"kind": 3}, ["Enum"]):
            pin, error = row_cpp_pin({"cpp_selector": selector})
            self.assertIsNone(pin)
            self.assertIn(repr(selector), error)

    def test_blank_declaration_id_is_an_error(self) -> None:
        pin, error = row_cpp_pin({"cpp_declaration_id": ""})
        self.assertIsNone(pin)
        self.assertIn("cpp_declaration_id ''", error)

    def test_signature_compares_after_whitespace_normalization(self) -> None:
        record = declaration("a", "F", "FunctionTemplate", NODE,
                             signature="decltype(x.Nb() , bool()) (const T &) const")
        pin = CppPin(None, {"signature": "decltype(x.Nb(), bool())(const T&) const"})
        self.assertEqual(pinned_candidates(pin, [record]), [record])

    def test_signature_variants_also_match(self) -> None:
        record = declaration("a", "T", "TypeAlias", PROPS, signature="class V<double>",
                             signature_variants=["class V<double>", "class V<>"])
        self.assertEqual(pinned_candidates(CppPin(None, {"signature": "class V<>"}), [record]), [record])


class SelectorResolutionTest(unittest.TestCase):
    def setUp(self) -> None:
        self.template = declaration("tpl", "D::NodeTraits", "ClassTemplate", ITER)
        self.vertex = declaration("vtx", "D::NodeTraits", "ClassTemplateSpecialization", ITER)
        self.edge = declaration("edg", "D::NodeTraits", "ClassTemplateSpecialization", ITER)
        self.enum = declaration("enm", "P::ObjType", "Enum", PROPS)
        self.typedef = declaration("tdf", "P::ObjType", "Typedef", PROPS)
        self.using = declaration("use", "B::Box", "Using", BVH, api_resolution="unresolved_using_value_target")
        self.method = declaration("box", "B::Box", "CXXMethod", BVH, signature="int (int) const",
                                  parameter_types=["int"], written_parameter_types=["int"])
        records = [self.template, self.vertex, self.edge, self.enum, self.typedef, self.using, self.method]
        self.by_name: dict[str, list[JSONObject]] = {}
        for record in records:
            self.by_name.setdefault(str(record["qualified_name"]), []).append(record)

    def resolve(self, row: JSONObject) -> JSONObject:
        return match_api_row(row, self.by_name, {}, {ITER, PROPS, BVH})

    def test_one_header_specializations_stay_ambiguous_without_a_pin(self) -> None:
        self.assertEqual(self.resolve(ledger_row("D::NodeTraits", ITER))["resolution"], "ambiguous_cpp")

    def test_usr_pins_one_specialization(self) -> None:
        result = self.resolve(ledger_row("D::NodeTraits", ITER, cpp_selector={"usr": "c:@vtx"}))
        self.assertEqual((result["cpp_ids"], result["resolution"]), (["vtx"], "excluded"))

    def test_kind_pins_the_primary_template(self) -> None:
        result = self.resolve(ledger_row("D::NodeTraits", ITER, cpp_selector={"kind": "ClassTemplate"}))
        self.assertEqual(result["cpp_ids"], ["tpl"])

    def test_kind_splits_an_enum_from_its_typedef(self) -> None:
        enum = self.resolve(ledger_row("P::ObjType", PROPS, cpp_selector={"kind": "Enum"}))
        alias = self.resolve(ledger_row("P::ObjType", PROPS, cpp_selector={"kind": "Typedef"}))
        self.assertEqual((enum["cpp_ids"], alias["cpp_ids"]), (["enm"], ["tdf"]))

    def test_pinned_method_ignores_a_same_name_using_declaration(self) -> None:
        unpinned = self.resolve(ledger_row("B::Box(int) const", BVH))
        pinned = self.resolve(ledger_row("B::Box(int) const", BVH, cpp_selector={"kind": "CXXMethod"}))
        self.assertEqual(unpinned["resolution"], "unresolved_cpp_member_target")
        self.assertEqual(pinned["cpp_ids"], ["box"])

    def test_using_declaration_is_claimable_by_kind(self) -> None:
        result = self.resolve(ledger_row("B::Box", BVH, cpp_selector={"kind": "Using"}))
        self.assertEqual(result["resolution"], "unresolved_cpp_member_target")
        self.assertEqual(result["cpp_ids"], ["use"])

    def test_declaration_id_pins_exactly_and_a_stale_id_is_missing(self) -> None:
        exact = self.resolve(ledger_row("D::NodeTraits", ITER, cpp_declaration_id="edg"))
        stale = self.resolve(ledger_row("D::NodeTraits", ITER, cpp_declaration_id="gone"))
        self.assertEqual(exact["cpp_ids"], ["edg"])
        self.assertEqual(stale["cpp_status"], "missing")

    def test_pin_matching_two_declarations_stays_ambiguous(self) -> None:
        row = ledger_row("D::NodeTraits", ITER, cpp_selector={"kind": "ClassTemplateSpecialization"})
        self.assertEqual(self.resolve(row)["resolution"], "ambiguous_cpp")

    def test_malformed_pin_is_its_own_resolution(self) -> None:
        result = self.resolve(ledger_row("D::NodeTraits", ITER, cpp_selector={"bogus": "x"}))
        self.assertEqual(result["resolution"], "invalid_cpp_selector")

    def test_cpp_candidates_without_pin_keep_previous_behavior(self) -> None:
        status, matches = cpp_candidates("P::ObjType", "TK", self.by_name)
        self.assertEqual((status, len(matches)), ("ambiguous", 2))


if __name__ == "__main__":
    unittest.main()
