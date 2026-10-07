"""Category rules that separate OCCT boilerplate from API that needs Rust code."""

from __future__ import annotations

import unittest

from api_gap_categories import declaration_category


def declaration(name: str, kind: str = "CXXMethod", **fields: object) -> dict[str, object]:
    """One declaration-table record with the fields the classifier reads."""
    record: dict[str, object] = {"qualified_name": name, "kind": kind, "access": "public"}
    record.update(fields)
    return record


def spelled_in(header: str) -> dict[str, object]:
    return {"macro_generated": True, "spelling_location": {"file": "src/x/" + header}}


class MacroCategoryTest(unittest.TestCase):
    def test_allocation_macro(self) -> None:
        record = declaration("Geom_Line::operator new", **spelled_in("Standard_DefineAlloc.hxx"))
        self.assertEqual(declaration_category(record), "alloc-macro")

    def test_rtti_macro(self) -> None:
        record = declaration("Geom_Line::DynamicType", **spelled_in("Standard_Type.hxx"))
        self.assertEqual(declaration_category(record), "rtti-macro")

    def test_exception_constructor_spelled_with_class_token(self) -> None:
        record = declaration(
            "Geom2d_UndefinedValue::Geom2d_UndefinedValue", "CXXConstructor",
            parameter_types=["const char *", "const char *"], **spelled_in("Geom2d_UndefinedValue.hxx"),
        )
        self.assertEqual(declaration_category(record), "exception-macro")

    def test_other_macro_output_is_api(self) -> None:
        record = declaration("BRepGraphInc_Storage::Removed", **spelled_in("BRepGraphInc_Storage.lxx"))
        self.assertEqual(declaration_category(record), "member")


class SpecialMemberTest(unittest.TestCase):
    def test_destructor_and_assignment(self) -> None:
        self.assertEqual(declaration_category(declaration("A::~A", "CXXDestructor")), "special-member")
        self.assertEqual(declaration_category(declaration("A::operator=")), "special-member")

    def test_copy_and_move_constructors(self) -> None:
        for parameter in ("const class A &", "class A &&", "const A<T> &"):
            record = declaration("A::A", "CXXConstructor", parameter_types=[parameter])
            self.assertEqual(declaration_category(record), "special-member", parameter)

    def test_converting_constructor_is_api(self) -> None:
        record = declaration("A::A", "CXXConstructor", parameter_types=["const class B &"])
        self.assertEqual(declaration_category(record), "member")

    def test_string_constructor_outside_macro_is_api(self) -> None:
        record = declaration(
            "TCollection_AsciiString::TCollection_AsciiString", "CXXConstructor",
            parameter_types=["const char *"],
        )
        self.assertEqual(declaration_category(record), "member")


class PlainCategoryTest(unittest.TestCase):
    def test_kinds(self) -> None:
        cases = {
            "EnumConstant": "enum-constant", "Field": "field", "Typedef": "alias",
            "CXXRecord": "record", "ClassTemplate": "record", "Function": "member",
        }
        for kind, expected in cases.items():
            self.assertEqual(declaration_category(declaration("A::x", kind)), expected, kind)

    def test_protected_method(self) -> None:
        record = declaration("A::Step", access="protected")
        self.assertEqual(declaration_category(record), "protected-member")


if __name__ == "__main__":
    unittest.main()
