"""Selector rules stay exact when class templates and overloads are involved."""

import unittest
import json
from pathlib import Path

from api_crosswalk import cpp_candidates, parameter_match
from api_inventory.inputs import JSONObject
from api_template_selectors import prefer_nonconst_overload


def row(name: str, signature: str, params: list[str], kind: str = "CXXMethod") -> JSONObject:
    return {
        "id": f"{name}:{signature}",
        "qualified_name": name,
        "kind": kind,
        "signature": signature,
        "parameter_types": params,
        "written_parameter_types": params,
        "api_visible": True,
        "_toolkits": {"TKFixture"},
    }


def indexed(*rows: JSONObject) -> dict[str, list[JSONObject]]:
    result: dict[str, list[JSONObject]] = {}
    for declaration in rows:
        result.setdefault(declaration["qualified_name"], []).append(declaration)
    return result


class TemplateSelectorTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        fixture = Path(__file__).parent / "fixtures/cpp-template-selector-declarations.json"
        cls.document = json.loads(fixture.read_text(encoding="utf-8"))
        cls.rows = cls.document["declarations"]
        for declaration in cls.rows:
            declaration["_toolkits"] = {"TKernel"}
        cls.by_name = indexed(*cls.rows)

    def test_compiler_fixture_records_pinned_sources(self) -> None:
        provenance = self.document["provenance"]
        self.assertEqual(provenance["occt_revision"], "3d097a0328e71b826377d4814ab05ec3c3d23871")
        headers = provenance["headers"]
        self.assertEqual(
            headers["src/FoundationClasses/TKernel/NCollection/NCollection_Vec2.hxx"],
            "3cd5f42867a7ad41387beb25df16033e3cad11c1b85f2a90c460bd84815626b9",
        )
        self.assertEqual(
            headers["src/FoundationClasses/TKernel/Standard/Standard_ArrayStreamBuffer.hxx"],
            "a12154585fee20870cdb197435c92cf02ca715f33e2a30433700a14e057db6f4",
        )
        self.assertIn("NCollection_Vec2::x", {row["qualified_name"] for row in self.rows})

    def test_compiler_template_constructor_matches_exact_written_parameter(self) -> None:
        identity = "NCollection_Vec2::NCollection_Vec2(const Element_t)"
        status, selected = cpp_candidates(identity, "TKernel", self.by_name)
        self.assertEqual(status, "unique")
        self.assertEqual(selected[0]["written_parameter_types"], ["const Element_t"])

    def test_compiler_swizzle_const_noexcept_is_selected(self) -> None:
        status, selected = cpp_candidates("NCollection_Vec2::x() const", "TKernel", self.by_name)
        self.assertEqual(status, "unique")
        self.assertIn("const noexcept", selected[0]["signature"])

    def test_compiler_stream_written_type_spellings_match(self) -> None:
        seekoff = next(row for row in self.rows if row["qualified_name"].endswith("::seekoff"))
        selector = ["std::basic_streambuf<char>::off_type", "std::ios_base::seekdir", "std::ios_base::openmode"]
        self.assertTrue(parameter_match(selector, seekoff))

    def test_compiler_stream_seekpos_written_type_spellings_match(self) -> None:
        seekpos = next(row for row in self.rows if row["qualified_name"].endswith("::seekpos"))
        selector = ["std::basic_streambuf<char>::pos_type", "std::ios_base::openmode"]
        self.assertTrue(parameter_match(selector, seekpos))

    def test_namespaced_record_constructor_shorthand_resolves(self) -> None:
        record = row("sample::Record", "Record", [], "CXXRecord")
        ctor = row("sample::Record::Record", "Record()", [], "CXXConstructor")
        status, selected = cpp_candidates("sample::Record()", "TKFixture", indexed(record, ctor))
        self.assertEqual(status, "unique")
        self.assertEqual(selected, [ctor])

    def test_namespaced_class_template_constructor_shorthand_resolves(self) -> None:
        template = row("sample::Box", "Box<T>", [], "ClassTemplate")
        template["usr"] = "template:sample::Box"
        ctor = row("sample::Box::Box", "Box()", [], "CXXConstructor")
        ctor["parent_usr"] = template["usr"]
        status, selected = cpp_candidates("sample::Box()", "TKFixture", indexed(template, ctor))
        self.assertEqual(status, "unique")
        self.assertEqual(selected, [ctor])

    def test_name_only_const_pair_stays_ambiguous(self) -> None:
        name = "sample::Vector::x"
        mutable = row(name, "double () noexcept", [])
        immutable = row(name, "double () const noexcept", [])
        status, selected = cpp_candidates("sample::Vector::x", "TKFixture", indexed(mutable, immutable))
        self.assertEqual(status, "ambiguous")
        self.assertEqual(len(selected), 2)

    def test_explicit_empty_parameter_list_selects_nonconst_pair_member(self) -> None:
        name = "sample::Vector::x"
        mutable = row(name, "double () noexcept", [])
        immutable = row(name, "double () const noexcept", [])
        status, selected = cpp_candidates("sample::Vector::x()", "TKFixture", indexed(mutable, immutable))
        self.assertEqual(status, "unique")
        self.assertEqual(selected, [mutable])

    def test_distinct_parameter_overloads_are_never_narrowed(self) -> None:
        name = "sample::Vector::scale"
        rows = [
            row(name, "void (double)", ["double"]),
            row(name, "void (int)", ["int"]),
        ]
        self.assertEqual(prefer_nonconst_overload(rows), rows)


if __name__ == "__main__":
    unittest.main()
