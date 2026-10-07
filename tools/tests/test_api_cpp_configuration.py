"""Rows for code the Linux capture does not compile resolve only with capture evidence."""

import unittest

from api_coverage_report import count_inactive_configuration
from api_cpp_configuration import (
    ConfigurationEvidence,
    configuration_claim,
    configuration_evidence,
    configured_cpp_status,
)
from api_crosswalk import match_api_row, resolve_status

ATOMIC = "src/FoundationClasses/TKernel/Standard/Standard_Atomic.hxx"
WNT = "src/FoundationClasses/TKernel/OSD/OSD_WNT.hxx"


def skipped(path: str, first: int, last: int) -> dict:
    return {"start": {"file": path, "line": first}, "end": {"file": path, "line": last}}


def configured_row(upstream_id: str, source: str, line: object, status: str = "excluded") -> dict:
    return {"upstream_id": upstream_id, "toolkit": "TKernel", "source": source,
            "status": status, "notes": "fixture",
            "cpp_configuration": {"condition": "defined(_WIN32)", "line": line}}


class EvidenceTest(unittest.TestCase):
    def setUp(self) -> None:
        headers = [{"path": WNT, "status": "parse_failed"}, {"path": ATOMIC, "status": "parsed"}]
        self.evidence = configuration_evidence([skipped(ATOMIC, 83, 200), {"start": 1}], headers)

    def test_skipped_range_and_unparsed_header_prove_inactivity(self) -> None:
        self.assertTrue(self.evidence.proves_inactive(ATOMIC, 83))
        self.assertTrue(self.evidence.proves_inactive(ATOMIC, 200))
        self.assertTrue(self.evidence.proves_inactive(WNT, 1))

    def test_compiled_lines_are_not_inactive(self) -> None:
        self.assertFalse(self.evidence.proves_inactive(ATOMIC, 82))
        self.assertFalse(self.evidence.proves_inactive("src/Other.hxx", 90))

    def test_claim_shape_is_validated_with_the_value(self) -> None:
        self.assertEqual(configuration_claim(configured_row("x", ATOMIC, 90)), (90, None))
        for line in (0, True, "90"):
            line_value, error = configuration_claim(configured_row("x", ATOMIC, line))
            self.assertIsNone(line_value)
            self.assertIn("cpp_configuration", error)

    def test_status_follows_the_evidence(self) -> None:
        inside, outside = configured_row("x", ATOMIC, 90), configured_row("x", ATOMIC, 20)
        self.assertEqual(configured_cpp_status(inside, "missing", self.evidence), "inactive_configuration")
        self.assertEqual(configured_cpp_status(outside, "missing", self.evidence), "unverified_configuration")
        self.assertEqual(configured_cpp_status(inside, "unique", self.evidence), "configuration_conflict")
        self.assertEqual(configured_cpp_status(inside, "missing", None), "unverified_configuration")
        self.assertEqual(configured_cpp_status({"upstream_id": "x"}, "missing", None), "missing")


class ResolutionTest(unittest.TestCase):
    def setUp(self) -> None:
        self.evidence = ConfigurationEvidence({ATOMIC: [(83, 200)]}, frozenset({WNT}))

    def resolve(self, row: dict) -> dict:
        return match_api_row(row, {}, {}, {ATOMIC, WNT}, self.evidence)

    def test_excluded_windows_row_is_accounted(self) -> None:
        result = self.resolve(configured_row("Standard_Atomic::_InterlockedIncrement(volatile long*)", ATOMIC, 90))
        self.assertEqual((result["cpp_status"], result["resolution"]), ("inactive_configuration", "excluded"))
        self.assertEqual(result["cpp_ids"], [])

    def test_implemented_row_still_needs_its_rust_target(self) -> None:
        row = configured_row("OSD_WNT::AdminSid()", WNT, 40, "implemented")
        row["rust_item"] = "rustycad_tkernel::osd::admin_sid"
        self.assertEqual(self.resolve(row)["resolution"], "missing_rust_target")

    def test_unverified_and_invalid_claims_stay_unresolved(self) -> None:
        self.assertEqual(self.resolve(configured_row("A::f()", ATOMIC, 10))["resolution"], "unverified_configuration")
        self.assertEqual(self.resolve(configured_row("A::f()", ATOMIC, -1))["resolution"], "invalid_cpp_configuration")

    def test_inactive_counts_like_a_unique_match(self) -> None:
        self.assertEqual(resolve_status("partial", "inactive_configuration", "unique"), "partial")
        self.assertEqual(resolve_status("excluded", "configuration_conflict", "unique"), "configuration_conflict")

    def test_report_counts_inactive_rows(self) -> None:
        rows = [{"cpp_status": "inactive_configuration"}, {"cpp_status": "unique"}]
        self.assertEqual(count_inactive_configuration(rows), 1)


if __name__ == "__main__":
    unittest.main()
