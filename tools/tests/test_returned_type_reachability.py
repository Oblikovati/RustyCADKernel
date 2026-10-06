"""Fail-closed merge checks for inferred private return-type reachability."""

from __future__ import annotations

import copy
import pathlib
import unittest

from api_inventory.inputs import JSONObject, JSONValue
from api_inventory.merge import (
    InventoryRecords,
    consume_event,
    declaration_counts,
    declaration_identity,
    verify_capture_summary,
)
from api_inventory.returned_reachability import POLICY, bind_returned_edge
from merge_occt_inventory import inventory_tables


def nested_declaration(name: str, kind: str = "CXXRecord") -> JSONObject:
    """Build a declaration with stable inventory identity for merger unit tests."""
    return {
        "event": "declaration",
        "usr": name,
        "qualified_name": name,
        "kind": kind,
        "source": "/fixture/Private.hxx",
        "api_visible": False,
        "signature": "int ()",
        "exclusion_reason": "private_ancestor",
    }


def returned_edge() -> JSONObject:
    """Build a candidate edge whose nested identities use the production algorithm."""
    return {
        "event": "returned_type_reachability",
        "source": "/fixture/Private.hxx",
        "seed_declaration": nested_declaration("Owner::get", "CXXMethod"),
        "producer_declaration": nested_declaration("Owner::get", "CXXMethod"),
        "target_declaration": nested_declaration("Owner::Hidden"),
        "members": [
            {
                "access_basis": "inferred_type_public_member",
                "declaration": nested_declaration("Owner::Hidden::read", "CXXMethod"),
            }
        ],
        "type_wrappers": ["pointer"],
        "status": "private_record_members_recorded",
        "coverage_credit": False,
        "reachability_complete": False,
    }


def compiler_summary(count: JSONValue = 1) -> JSONObject:
    """Return typed fixture metadata, permitting invalid JSON count tests."""
    return {
        "event": "summary",
        "complete_for_emit_scope": True,
        "alias_reachability_events": 0,
        "returned_type_reachability_policy": POLICY,
        "returned_type_reachability_events": count,
    }


class ReturnedTypeMergeTests(unittest.TestCase):
    def test_edges_do_not_promote_primary_declarations(self) -> None:
        records = InventoryRecords()
        consume_event(records, returned_edge(), "Private.hxx", pathlib.Path("/fixture"))
        self.assertEqual(declaration_counts(records)["api_declarations"], 0)
        edge = records.returned_type_reachability[0]
        self.assertEqual(
            edge["target_declaration"]["id"],
            declaration_identity(nested_declaration("Owner::Hidden")),
        )
        self.assertEqual(edge["members"][0]["declaration"]["source"], "Private.hxx")
        self.assertEqual(
            edge["members"][0]["declaration"]["id"],
            declaration_identity(
                nested_declaration("Owner::Hidden::read", "CXXMethod")
            ),
        )
        self.assertFalse(records.unsupported)
        self.assertEqual(
            inventory_tables(records, [], pathlib.Path("/fixture"))[
                "returned-type-reachability"
            ],
            [edge],
        )

    def test_overloads_keep_distinct_member_ids(self) -> None:
        first = returned_edge()
        second = copy.deepcopy(first)
        second["members"][0]["declaration"]["signature"] = "int (int)"
        bind_returned_edge(first, declaration_identity)
        bind_returned_edge(second, declaration_identity)
        self.assertNotEqual(
            first["members"][0]["declaration"]["id"],
            second["members"][0]["declaration"]["id"],
        )

    def test_missing_or_malformed_declarations_rejected(self) -> None:
        for field in ["seed_declaration", "producer_declaration", "target_declaration"]:
            event = returned_edge()
            event[field] = "not a declaration"
            with (
                self.subTest(field=field),
                self.assertRaisesRegex(
                    ValueError, "expected (nested|target) declaration"
                ),
            ):
                bind_returned_edge(event, declaration_identity)
        for members in ["wrong", ["wrong"], [{}]]:
            event = returned_edge()
            event["members"] = members
            with (
                self.subTest(members=members),
                self.assertRaises((ValueError, TypeError)),
            ):
                bind_returned_edge(event, declaration_identity)

    def test_unresolved_status_never_enumerates_members(self) -> None:
        event = returned_edge()
        event["status"] = "unsupported_template_or_dependent_type"
        with self.assertRaisesRegex(ValueError, "expected no enumerated members"):
            bind_returned_edge(event, declaration_identity)
        del event["members"]
        del event["target_declaration"]
        bind_returned_edge(event, declaration_identity)
        event["status"] = "claimed_complete"
        with self.assertRaisesRegex(ValueError, "expected supported"):
            bind_returned_edge(event, declaration_identity)

    def test_recorded_members_cannot_omit_target_or_member_list(self) -> None:
        for field in ["target_declaration", "members"]:
            event = returned_edge()
            del event[field]
            with self.subTest(field=field), self.assertRaises((ValueError, TypeError)):
                bind_returned_edge(event, declaration_identity)
        event = returned_edge()
        event["members"][0]["access_basis"] = "private_promoted"
        with self.assertRaisesRegex(ValueError, "supported access basis"):
            bind_returned_edge(event, declaration_identity)

    def test_credit_and_closure_cannot_be_promoted(self) -> None:
        for field in ["coverage_credit", "reachability_complete"]:
            event = returned_edge()
            event[field] = True
            with (
                self.subTest(field=field),
                self.assertRaisesRegex(ValueError, "expected withheld"),
            ):
                bind_returned_edge(event, declaration_identity)

    def test_count_and_policy_fail_closed(self) -> None:
        for count in [0, 2, True, "1", None]:
            records = InventoryRecords(
                returned_type_reachability=[returned_edge()],
                summaries=[compiler_summary(count)],
            )
            verify_capture_summary(records, 0, 0, "Private.hxx")
            self.assertEqual(
                records.unsupported[0]["reason"],
                "returned_type_reachability_count_mismatch",
            )
        summary = compiler_summary()
        del summary["returned_type_reachability_policy"]
        records = InventoryRecords(summaries=[summary])
        verify_capture_summary(records, 0, 0, "Private.hxx")
        self.assertEqual(
            records.unsupported[0]["reason"],
            "returned_type_reachability_capture_unavailable",
        )

    def test_summary_scope_counts_and_absent_summary(self) -> None:
        records = InventoryRecords(
            returned_type_reachability=[returned_edge(), returned_edge()],
            summaries=[compiler_summary()],
        )
        verify_capture_summary(records, 0, 0, "Private.hxx", previous_returned=1)
        self.assertFalse(records.unsupported)
        records.summaries.clear()
        verify_capture_summary(records, 0, 0, "Private.hxx")
        self.assertEqual(
            records.unsupported[0]["reason"], "incomplete_compiler_summary"
        )


if __name__ == "__main__":
    unittest.main()
