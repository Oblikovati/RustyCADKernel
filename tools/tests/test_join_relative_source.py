"""Relative oracle paths must retain the same freshness checks as absolute paths."""

from __future__ import annotations

import pathlib
import unittest
from unittest.mock import patch

from api_inventory.inputs import JSONObject
from join_legacy_runtime_evidence import checked_inventory


class FakeNativeInventory:
    """Record paths passed to the filesystem boundary; e.g. `fake.snapshot(source)`."""

    def __init__(self, status: str = "fresh") -> None:
        self.status: str = status
        self.assessed: list[pathlib.Path] = []
        self.scanned: list[pathlib.Path] = []

    def assess(
        self, report: JSONObject, root: pathlib.Path, source: pathlib.Path
    ) -> JSONObject:
        self.assessed.append(source)
        return {"status": self.status, "reasons": ["fixture freshness"]}

    def snapshot(self, source: pathlib.Path) -> JSONObject:
        self.scanned.append(source)
        return {"revision": "pinned-native-revision"}


class RelativeSourceJoinTest(unittest.TestCase):
    def test_relative_source_is_normalized_before_both_checks(self) -> None:
        fake = FakeNativeInventory()
        source = pathlib.Path("../OCCT")
        with (
            patch("join_legacy_runtime_evidence.assess_report", fake.assess),
            patch("join_legacy_runtime_evidence.snapshot", fake.snapshot),
        ):
            result = checked_inventory(
                {"revision": "pinned-native-revision"}, pathlib.Path.cwd(), source
            )
        self.assertEqual(result["revision"], "pinned-native-revision")
        self.assertEqual(fake.assessed, [source.resolve()])
        self.assertEqual(fake.scanned, [source.resolve()])

    def test_relative_source_does_not_bypass_stale_report_rejection(self) -> None:
        fake = FakeNativeInventory("stale")
        with (
            patch("join_legacy_runtime_evidence.assess_report", fake.assess),
            patch("join_legacy_runtime_evidence.snapshot", fake.snapshot),
        ):
            with self.assertRaisesRegex(ValueError, "runtime report is stale"):
                checked_inventory({}, pathlib.Path.cwd(), pathlib.Path("../OCCT"))
        self.assertEqual(fake.scanned, [])


if __name__ == "__main__":
    unittest.main()
