"""Keep source-discovery output stable when composing the status report."""

from __future__ import annotations

import unittest

from api_inventory.inputs import JSONObject
from porting import _source_inventory_lines


def source_discovery_fixture() -> JSONObject:
    """Supply discovery counts independently of the API and test ledgers."""
    return {
        "toolkits": [
            {
                "module": "Foundation",
                "name": "TKA",
                "packages": ["A", "B"],
                "header_candidates": ["A.hxx", "B.hxx", "C.hxx"],
                "gtest_sources": ["Test.cxx"],
                "gtest_declaration_candidates": [
                    {"name": "Suite.A"},
                    {"name": "Suite.B"},
                ],
            }
        ],
        "draw_case_candidates": ["case1", "case2", "case3", "case4"],
        "upstream_test_files": ["case1", "case2", "case3", "case4", "support"],
    }


class PortingSourceStatusTest(unittest.TestCase):
    def test_discovery_counts_preserve_their_labels_and_candidate_boundary(
        self,
    ) -> None:
        self.assertEqual(
            _source_inventory_lines(source_discovery_fixture()),
            [
                "Foundation: 1 toolkits",
                "Total: 1 toolkits; 2 packages",
                "Recursive source header candidates (not the installed API boundary): 3",
                "GoogleTest source files: 1",
                "GoogleTest declaration candidates: 2",
                "DRAW case candidates: 4",
                "All regression files (including support): 5",
            ],
        )


if __name__ == "__main__":
    unittest.main()
