"""Row builders: each claim needs its check, and missing evidence leaves the item open."""

import unittest

from api_macro_classes import RustClass
from api_macro_rows import SpecialMember, special_row
from api_macro_rtti_rows import FAILURE_TYPE, MacroEvidence, exception_row
from api_macro_rust import CLONE, RustInventoryIndex, trait_item
from api_macro_source import DEFAULTED, DELETED, USER_BODY
from macro_fakes import FakeRustSources, declaration, rust_document, rust_item

POINT = "rustycad_tkmath::gp::Point2"
FACTORY = "rustycad_tkgeombase::lprop::not_defined"


def member(kind: str, form: str, rust: str | None = POINT, name: str = "gp_Pnt2d::gp_Pnt2d") -> SpecialMember:
    row = declaration("x", name, "CXXConstructor", "void (const class gp_Pnt2d &)", ["const gp_Pnt2d &"],
                      "src/FoundationClasses/TKMath/gp/gp_Pnt2d.hxx")
    rust_class = RustClass(rust, False, "ledger record row", None) if rust else None
    return SpecialMember(row, kind, form, "TKMath", rust_class)


def clone_index(source: str = "crates/tkmath/src/gp/point.rs") -> RustInventoryIndex:
    return RustInventoryIndex(rust_document([rust_item(trait_item(POINT, CLONE, "clone"), "function", source)], "rustycad_tkmath"))


class SpecialRowTests(unittest.TestCase):
    def test_defaulted_copy_with_derived_clone_is_implemented(self) -> None:
        tree = FakeRustSources({"crates/tkmath/src/gp/point.rs": "#[derive(Clone, Copy)]\npub struct Point2 { x: f64 }\n"})
        row, reason = special_row(member("copy_constructor", DEFAULTED), clone_index(), tree)
        self.assertIsNone(reason)
        self.assertEqual((row["status"], row["rust_item"]), ("implemented", trait_item(POINT, CLONE, "clone")))

    def test_defaulted_copy_with_manual_clone_is_partial(self) -> None:
        tree = FakeRustSources({"crates/tkmath/src/gp/point.rs": "pub struct Point2;\nimpl Clone for Point2 {}\n"})
        row, _ = special_row(member("copy_assignment", DEFAULTED), clone_index(), tree)
        self.assertEqual(row["status"], "partial")

    def test_copy_without_clone_or_with_user_body_stays_open(self) -> None:
        empty = RustInventoryIndex(rust_document([]))
        self.assertEqual(special_row(member("copy_constructor", DEFAULTED), empty, FakeRustSources({}))[1],
                         "copy_constructor_without_rust_clone")
        self.assertEqual(special_row(member("copy_constructor", USER_BODY), empty, FakeRustSources({}))[1],
                         "copy_constructor_user_body")

    def test_deleted_copy_reports_an_extra_rust_clone(self) -> None:
        row, _ = special_row(member("copy_constructor", DELETED), clone_index(), FakeRustSources({}))
        self.assertEqual(row["status"], "excluded")
        self.assertIn("implements Clone", row["notes"])

    def test_destructor_with_body_needs_review(self) -> None:
        index = RustInventoryIndex(rust_document([]))
        row, reason = special_row(member("destructor", USER_BODY), index, FakeRustSources({}))
        self.assertIsNone(row)
        self.assertEqual(reason, "destructor_user_body")

    def test_allocation_without_rust_type_is_excluded_without_target(self) -> None:
        alloc = declaration("a", "gp_Pnt2d::operator new", "CXXMethod", "void *(unsigned long)", ["size_t"],
                            spelled=("src/FoundationClasses/TKernel/Standard/Standard_DefineAlloc.hxx", 55, 9), macro=True)
        row, _ = special_row(SpecialMember(alloc, "allocation", DEFAULTED, "TKMath", None),
                             RustInventoryIndex(rust_document([])), FakeRustSources({}))
        self.assertEqual((row["status"], row["rust_item"]), ("excluded", None))
        self.assertIn("DEFINE_STANDARD_ALLOC", row["notes"])


class ExceptionRowTests(unittest.TestCase):
    def evidence(self, pairs: set[tuple[str, str]]) -> MacroEvidence:
        return MacroEvidence(RustInventoryIndex(rust_document([])), {}, {}, {},
                             {"LProp_NotDefined": [FACTORY]}, {"LProp_NotDefined": "Standard_DomainError"}, pairs)

    def exception(self, kind: str, parameters: list[str]) -> SpecialMember:
        row = declaration("e", "LProp_NotDefined::LProp_NotDefined", "CXXConstructor", "", parameters,
                          "src/ModelingData/TKGeomBase/LProp/LProp_NotDefined.hxx", macro=True)
        return SpecialMember(row, kind, DEFAULTED, "TKGeomBase", None)

    def test_exception_type_pairs_failure_with_factory(self) -> None:
        row, _ = exception_row(self.exception("exception_type", []), self.evidence(set()))
        self.assertEqual(row["rust_items"], [FAILURE_TYPE, FACTORY])

    def test_constructor_is_partial_without_hierarchy_pair(self) -> None:
        member_row = self.exception("exception_constructor", ["const char *"])
        partial, _ = exception_row(member_row, self.evidence(set()))
        complete, _ = exception_row(member_row, self.evidence({("LProp_NotDefined", "Standard_DomainError")}))
        self.assertEqual((partial["status"], complete["status"]), ("partial", "implemented"))


if __name__ == "__main__":
    unittest.main()
