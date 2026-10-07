"""End-to-end generation over a one-class fixture: checks, crosswalk round trip, idempotence."""

import copy
import json
import unittest

from api_macro_ledger import GeneratorInputs, MacroRowGenerator
from api_macro_rust import RUNTIME_TYPED, TRANSIENT, trait_item
from macro_fakes import REGISTRATION_MACROS, FakeOcctSources, FakeRustSources, declaration, header_manifest, rust_document, rust_item

HEADER = "src/ModelingData/TKG2d/Geom2d/Geom2d_Line.hxx"
TYPE_HXX = "src/FoundationClasses/TKernel/Standard/Standard_Type.hxx"
ALLOC_HXX = "src/FoundationClasses/TKernel/Standard/Standard_DefineAlloc.hxx"
LINE = "rustycad_tkg2d::geom2d::Geom2dLine"
OCCT_TEXT = "class Geom2d_Line\n{\n  Geom2d_Line(const Geom2d_Line&) = delete;\n  ~Geom2d_Line() = default;\n};\n"
RUST_TEXT = (
    'impl_transient!(Geom2dLine, "Geom2d_Line", dyn Geom2dCurve, base);\n'
    'impl_transient_interface!(dyn Geom2dCurve, "Geom2d_Curve", dyn Geom2dGeometry);\n'
)


def declarations() -> list[dict]:
    rtti = (TYPE_HXX, 70, 1)
    return [
        declaration("rec", "Geom2d_Line", "CXXRecord", header=HEADER),
        declaration("ctor", "Geom2d_Line::Geom2d_Line", "CXXConstructor", "void (const class gp_Lin2d &)", ["const gp_Lin2d &"], HEADER),
        declaration("new", "Geom2d_Line::operator new", "CXXMethod", "void *(unsigned long)", ["size_t"], HEADER, (ALLOC_HXX, 55, 9), True),
        declaration("name", "Geom2d_Line::get_type_name", "CXXMethod", "const char *()", [], HEADER, rtti, True),
        declaration("desc", "Geom2d_Line::get_type_descriptor", "CXXMethod", "const handle &()", [], HEADER, rtti, True),
        declaration("dyn", "Geom2d_Line::DynamicType", "CXXMethod", "const handle &() const", [], HEADER, rtti, True),
        {**declaration("base", "Geom2d_Line::base_type", "Typedef", "class Geom2d_Curve", None, HEADER, rtti, True), "parameter_types": None},
        declaration("copy", "Geom2d_Line::Geom2d_Line", "CXXConstructor", "void (const class Geom2d_Line &)", ["const class Geom2d_Line &"], HEADER, (HEADER, 3, 3)),
        declaration("dtor", "Geom2d_Line::~Geom2d_Line", "CXXDestructor", "void () noexcept", [], HEADER, (HEADER, 4, 3)),
    ]


def rust_inventory() -> dict:
    items = [
        rust_item(LINE, "struct"),
        rust_item(f"{LINE}::new", "function"),
        rust_item(trait_item(LINE, RUNTIME_TYPED, "TYPE_NAME"), "assoc_const", signature={"value": '"Geom2d_Line"'}),
        rust_item(trait_item(LINE, RUNTIME_TYPED, "static_type"), "function"),
        rust_item(trait_item(LINE, TRANSIENT, "dynamic_type"), "function"),
    ]
    return rust_document(items)


def ledger(extra: list[dict] | None = None) -> dict:
    rows = [
        {"upstream_id": "Geom2d_Line::Geom2d_Line(const gp_Lin2d&)", "toolkit": "TKG2d", "source": HEADER,
         "rust_item": f"{LINE}::new", "status": "implemented", "notes": "hand"},
        {"upstream_id": "Geom2d_Line::DEFINE_STANDARD_RTTIEXT", "toolkit": "TKG2d", "source": HEADER,
         "rust_item": f"{LINE}::static_type", "status": "implemented", "notes": "old evidence"},
    ]
    return {"schema_version": 1, "api": rows + (extra or []), "tests": []}


def run(parity: dict) -> dict:
    inputs = GeneratorInputs(
        declarations=declarations(),
        headers=header_manifest({HEADER: "TKG2d"}),
        rust=rust_inventory(),
        parity=copy.deepcopy(parity),
        occt=FakeOcctSources({HEADER: OCCT_TEXT}),
        rust_sources=FakeRustSources({"crates/tkernel/src/standard/transient.rs": REGISTRATION_MACROS, "crates/tkg2d/src/geom2d/line.rs": RUST_TEXT}),
    )
    return MacroRowGenerator(inputs).run().parity


def by_id(parity: dict) -> dict[str, dict]:
    return {row.get("cpp_declaration_id", row["upstream_id"]): row for row in parity["api"]}


class MacroRowGeneratorTests(unittest.TestCase):
    def test_every_member_gets_a_checked_row(self) -> None:
        rows = by_id(run(ledger()))
        self.assertEqual(rows["new"]["status"], "excluded")
        self.assertEqual(rows["new"]["rust_item"], LINE)
        self.assertEqual(rows["name"]["rust_item"], trait_item(LINE, RUNTIME_TYPED, "TYPE_NAME"))
        self.assertEqual([rows[key]["status"] for key in ("name", "desc", "dyn")], ["implemented"] * 3)
        self.assertEqual(rows["copy"]["status"], "excluded")
        self.assertIn("= delete", rows["copy"]["notes"])
        self.assertEqual(rows["dtor"]["status"], "excluded")
        self.assertEqual((rows["rec"]["upstream_id"], rows["rec"]["status"]), ("Geom2d_Line", "implemented"))

    def test_macro_row_is_retired_with_its_notes_kept(self) -> None:
        parity = run(ledger())
        identities = [row["upstream_id"] for row in parity["api"]]
        self.assertNotIn("Geom2d_Line::DEFINE_STANDARD_RTTIEXT", identities)
        self.assertIn("old evidence", by_id(parity)["desc"]["notes"])

    def test_rerun_is_idempotent(self) -> None:
        first = run(ledger())
        self.assertEqual(json.dumps(run(first)), json.dumps(first))

    def test_hand_row_wins_over_generation(self) -> None:
        hand = {"upstream_id": "Geom2d_Line::operator new(size_t)", "toolkit": "TKG2d", "source": HEADER,
                "rust_item": None, "status": "excluded", "notes": "hand allocation row"}
        rows = run(ledger([hand]))["api"]
        claims = [row for row in rows if row["upstream_id"] == "Geom2d_Line::operator new(size_t)"]
        self.assertEqual([row["notes"] for row in claims], ["hand allocation row"])

    def test_parent_mismatch_is_partial(self) -> None:
        inputs = declarations()
        parity = ledger()
        base = next(row for row in inputs if row["id"] == "base")
        base["signature"] = "class Geom2d_Conic"
        generator_inputs = GeneratorInputs(inputs, header_manifest({HEADER: "TKG2d"}), rust_inventory(), parity,
                                           FakeOcctSources({HEADER: OCCT_TEXT}),
                                           FakeRustSources({"crates/tkernel/src/standard/transient.rs": REGISTRATION_MACROS, "crates/tkg2d/src/geom2d/line.rs": RUST_TEXT}))
        rows = by_id(MacroRowGenerator(generator_inputs).run().parity)
        self.assertEqual(rows["desc"]["status"], "partial")
        self.assertIn("Geom2d_Curve", rows["desc"]["notes"])


if __name__ == "__main__":
    unittest.main()
