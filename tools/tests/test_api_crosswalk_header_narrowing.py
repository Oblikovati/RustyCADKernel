"""Same-name declarations split only by the declaring header a ledger row names."""

import unittest

from api_crosswalk import match_api_row, narrow_by_declaring_header
from api_inventory.inputs import JSONObject

GP = "src/FoundationClasses/TKMath/gp/gp_Pnt.hxx"
TOPLOC = "src/FoundationClasses/TKMath/TopLoc/TopLoc_Location.lxx"


def specialization(identity: str, source: str) -> JSONObject:
    """Build a `std::hash<X>` record fixture, e.g. `specialization("hash-pnt", GP)`."""
    return {
        "id": identity,
        "qualified_name": "std::hash",
        "kind": "ClassTemplateSpecialization",
        "signature": "",
        "source": source,
        "api_visible": True,
        "_toolkits": {"TKMath"},
    }


def excluded_row(source: str) -> JSONObject:
    return {"upstream_id": "std::hash", "toolkit": "TKMath", "source": source,
            "rust_item": None, "status": "excluded", "notes": "fixture"}


class HeaderNarrowingTest(unittest.TestCase):
    def setUp(self) -> None:
        self.gp = specialization("hash-pnt", GP)
        self.toploc = specialization("hash-location", TOPLOC)
        self.by_name = {"std::hash": [self.gp, self.toploc]}

    def test_row_header_selects_its_own_declaration(self) -> None:
        result = match_api_row(excluded_row(GP), self.by_name, {}, {GP})
        self.assertEqual(result["cpp_status"], "unique")
        self.assertEqual(result["cpp_ids"], ["hash-pnt"])
        self.assertEqual(result["resolution"], "excluded")

    def test_header_without_a_declaration_stays_ambiguous(self) -> None:
        other = "src/FoundationClasses/TKMath/Poly/Poly_MakeLoops.hxx"
        result = match_api_row(excluded_row(other), self.by_name, {}, {other})
        self.assertEqual(result["cpp_status"], "ambiguous")

    def test_two_declarations_in_one_header_stay_ambiguous(self) -> None:
        twin = specialization("hash-pnt-twin", GP)
        status, matches = narrow_by_declaring_header("ambiguous", [self.gp, twin], GP)
        self.assertEqual(status, "ambiguous")
        self.assertEqual(len(matches), 2)

    def test_unique_and_missing_statuses_are_untouched(self) -> None:
        self.assertEqual(narrow_by_declaring_header("unique", [self.gp], TOPLOC), ("unique", [self.gp]))
        self.assertEqual(narrow_by_declaring_header("missing", [], GP), ("missing", []))

    def test_rows_without_a_source_keep_the_ambiguity(self) -> None:
        status, _ = narrow_by_declaring_header("ambiguous", [self.gp, self.toploc], None)
        self.assertEqual(status, "ambiguous")


if __name__ == "__main__":
    unittest.main()
