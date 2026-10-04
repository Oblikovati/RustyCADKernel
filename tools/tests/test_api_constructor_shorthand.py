"""Constructor shorthand resolves only to exact inventoried class members."""
import unittest
from api_crosswalk import cpp_candidates
from api_inventory.inputs import JSONObject


def declaration(name: str, kind: str, params: list[str]) -> JSONObject:
    return {"id": name + str(params), "qualified_name": name, "kind": kind,
            "written_parameter_types": params, "api_visible": True,
            "_toolkits": {"TKMath"}}


def catalog(name: str) -> dict[str, list[JSONObject]]:
    leaf = name.rsplit("::", 1)[-1]
    return {name: [declaration(name, "CXXRecord", [])],
            f"{name}::{leaf}": [declaration(f"{name}::{leaf}", "CXXConstructor", []),
                               declaration(f"{name}::{leaf}", "CXXConstructor", ["int"])]}


class ConstructorShorthandTest(unittest.TestCase):
    def test_exact_record_selects_constructor_overload(self) -> None:
        for name in ["Range", "sample::Range"]:
            for params in ["", "int"]:
                status, rows = cpp_candidates(f"{name}({params})", "TKMath", catalog(name))
                self.assertEqual(status, "unique")
                self.assertEqual(rows[0]["kind"], "CXXConstructor")
                self.assertEqual(rows[0]["written_parameter_types"], [params] if params else [])

    def test_bare_class_remains_record(self) -> None:
        status, rows = cpp_candidates("Range", "TKMath", catalog("Range"))
        self.assertEqual(status, "unique")
        self.assertEqual(rows[0]["kind"], "CXXRecord")

    def test_unavailable_overload_does_not_match_record(self) -> None:
        status, _ = cpp_candidates("Range(double)", "TKMath", catalog("Range"))
        self.assertEqual(status, "missing")

    def test_other_namespace_cannot_supply_constructor(self) -> None:
        status, _ = cpp_candidates("different::Range()", "TKMath", catalog("sample::Range"))
        self.assertEqual(status, "missing")

    def test_private_constructor_is_not_credited(self) -> None:
        entries = catalog("Range")
        entries["Range::Range"][0]["api_visible"] = False
        status, _ = cpp_candidates("Range()", "TKMath", entries)
        self.assertEqual(status, "missing")

    def test_function_and_constructor_remain_ambiguous(self) -> None:
        entries = catalog("Range")
        entries["Range"].append(declaration("Range", "Function", []))
        status, rows = cpp_candidates("Range()", "TKMath", entries)
        self.assertEqual(status, "ambiguous")
        self.assertEqual(len(rows), 2)

    def test_constructor_from_other_toolkit_is_not_credited(self) -> None:
        status, _ = cpp_candidates("Range()", "TKOther", catalog("Range"))
        self.assertEqual(status, "missing")
