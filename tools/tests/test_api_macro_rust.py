"""Rust-side evidence: inventory trait items, macro registrations and factories."""

import unittest

from api_macro_rust import CLONE, RUNTIME_TYPED, RustInventoryIndex, preferred_path, trait_item
from api_macro_rust_registrations import transient_registrations
from api_macro_rust_source import clone_is_derived, exception_factories, exception_parent_pairs
from macro_fakes import REGISTRATION_MACROS, FakeRustSources, rust_document, rust_item

LINE = "rustycad_tkg2d::geom2d::Geom2dLine"
SOURCE = """
impl_transient!(Geom2dLine, "Geom2d_Line", dyn Geom2dCurve, base);
impl_transient_interface!(
    dyn Geom2dCurve,
    "Geom2d_Curve",
    dyn Geom2dGeometry
);
macro_rules! x { ($c:ty) => { $crate::impl_transient!($c, $name, P, f); } }
/// Example: `impl_transient!(Doc, "Doc_Example", Parent, field)` is documentation only.
define_alert!(
    /// Native alert.
    AlertUnableToTrim, "BOPAlgo_AlertUnableToTrim"
);
#[derive(Clone, Debug)]
/// A point.
pub struct Point;
pub struct Manual;
impl Clone for Manual { fn clone(&self) -> Self { Manual } }
pub fn undefined_value(message: &str) -> Failure {
    Failure::with_type("Geom2d_UndefinedValue", message, "")
}
pub(crate) const PAIRS_HIERARCHY: &[(&str, &str)] = &[("Child_Error", "Standard_DomainError")];
"""


class RustInventoryIndexTests(unittest.TestCase):
    def test_type_name_owners_skip_the_facade(self) -> None:
        name = trait_item(LINE, RUNTIME_TYPED, "TYPE_NAME")
        facade = trait_item("rustycad_kernel::geom2d::Geom2dLine", RUNTIME_TYPED, "TYPE_NAME")
        items = [rust_item(path, "assoc_const", signature={"value": '"Geom2d_Line"'}) for path in (name, facade)]
        index = RustInventoryIndex(rust_document(items + [rust_item(LINE, "struct")]))
        self.assertEqual(index.type_name_owners, {"Geom2d_Line": [LINE]})
        self.assertTrue(index.is_type(LINE))

    def test_trait_item_lookup(self) -> None:
        index = RustInventoryIndex(rust_document([rust_item(trait_item(LINE, CLONE, "clone"), "function")]))
        self.assertTrue(index.has_trait_item(LINE, CLONE, "clone"))

    def test_preferred_path_keeps_the_toolkit_crate(self) -> None:
        paths = ["rustycad_tkg3d::a::T", "rustycad_tkg2d::a::b::T", "rustycad_tkg2d::a::T"]
        self.assertEqual(preferred_path(paths, "rustycad_tkg2d"), "rustycad_tkg2d::a::T")
        self.assertIsNone(preferred_path(["rustycad_a::T", "rustycad_b::T"], "rustycad_c"))


class RustSourceScanTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tree = FakeRustSources({"crates/tkernel/src/standard/transient.rs": REGISTRATION_MACROS,
                                     "crates/tkg2d/src/geom2d/line.rs": SOURCE})

    def test_registrations_with_literal_names_only(self) -> None:
        found = [item for item in transient_registrations(self.tree) if item.file.endswith("line.rs")]
        self.assertEqual([(item.cxx_name, item.parent_type, item.interface) for item in found], [
            ("Geom2d_Line", "dyn Geom2dCurve", False),
            ("Geom2d_Curve", "dyn Geom2dGeometry", True),
            ("BOPAlgo_AlertUnableToTrim", "AlertBase", False),
        ])
        self.assertEqual((found[0].line, found[2].macro), (2, "define_alert"))

    def test_derived_and_manual_clone(self) -> None:
        self.assertTrue(clone_is_derived(self.tree, "crates/tkg2d/src/geom2d/line.rs", "Point"))
        self.assertFalse(clone_is_derived(self.tree, "crates/tkg2d/src/geom2d/line.rs", "Manual"))

    def test_exception_factory_and_hierarchy_pairs(self) -> None:
        factories = exception_factories(self.tree)
        self.assertEqual([(item.function, item.cxx_name) for item in factories], [("undefined_value", "Geom2d_UndefinedValue")])
        self.assertIn(("Child_Error", "Standard_DomainError"), exception_parent_pairs(self.tree))


if __name__ == "__main__":
    unittest.main()
