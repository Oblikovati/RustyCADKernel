"""Exact reachability IDs and declaration sites remain separate from API coverage credit."""

from __future__ import annotations

import unittest

from api_inventory.inputs import JSONObject
from report_returned_type_inventory import (
    collect_members,
    compared_declaration,
    declaration_comparison,
    declaration_has_site,
    member_rows,
    nested_declaration,
    report_summary,
)


def native_declaration(identifier: str, signature: str = "int ()") -> JSONObject:
    """Build bound native records with fixed identities for pure reporting tests."""
    return {
        "id": identifier,
        "source": "Base.hxx",
        "kind": "CXXMethod",
        "signature": signature,
        "identity_signature": signature,
        "usr": identifier,
        "qualified_name": identifier,
        "api_visible": False,
        "exclusion_reason": "private_ancestor",
    }


def exposure(
    member: str = "method", access: str = "inferred_type_public_member"
) -> JSONObject:
    """Build an observed reachability edge without granting any coverage credit."""
    return {
        "id": "edge-" + member,
        "status": "private_record_members_recorded",
        "seed_declaration": native_declaration("seed"),
        "producer_declaration": native_declaration("producer"),
        "target_declaration": native_declaration("record"),
        "members": [
            {"access_basis": access, "declaration": native_declaration(member)}
        ],
    }


class ReturnedScopeReportTests(unittest.TestCase):
    def test_first_source_does_not_invent_new_api_ids(self) -> None:
        current = native_declaration("method")
        prior = {**current, "source": "Other.hxx"}
        comparison = declaration_comparison([current], [prior], {"Base.hxx"})
        self.assertEqual(comparison[0]["state"], "unchanged")
        prior["sites"] = [{"source": "Base.hxx"}]
        self.assertTrue(declaration_has_site(prior, {"Base.hxx"}))
        self.assertEqual(
            declaration_comparison([], [prior], {"Base.hxx"})[0]["state"],
            "only_published",
        )

    def test_changed_fields_and_scope_omissions_are_explicit(self) -> None:
        prior = native_declaration("method")
        current = {**prior, "api_visible": True}
        changed = declaration_comparison([current], [prior], {"Base.hxx"})[0]
        self.assertEqual(changed["changed_fields"], ["api_visible"])
        self.assertEqual(changed["state"], "changed")
        self.assertEqual(
            declaration_comparison([current], [], set())[0]["state"], "only_current"
        )
        self.assertFalse(declaration_has_site(prior, {"Other.hxx"}))

    def test_overloads_and_multiple_access_observations_remain_distinct(self) -> None:
        edges = [
            exposure(),
            exposure(access="requires_derived_inferred_type"),
            exposure("overload"),
        ]
        members = member_rows(edges, [native_declaration("method")])
        self.assertEqual(len(collect_members(edges)), 2)
        first = next(row for row in members if row["id"] == "method")
        self.assertEqual(first["observations"], 2)
        self.assertEqual(
            first["access_basis_observations"],
            ["inferred_type_public_member", "requires_derived_inferred_type"],
        )
        self.assertFalse(first["coverage_credit"])
        self.assertFalse(first["callability_proven"])
        self.assertTrue(first["primary_declaration_present"])
        self.assertFalse(
            next(row for row in members if row["id"] == "overload")[
                "primary_declaration_present"
            ]
        )
        summary = report_summary(edges, members, [])
        self.assertEqual(summary["distinct_returned_edges"], 2)
        self.assertEqual(summary["distinct_reached_private_record_types"], 1)
        self.assertFalse(summary["api_denominator_complete"])

    def test_comparison_requires_at_least_one_declaration(self) -> None:
        with self.assertRaisesRegex(ValueError, "expected current or published record"):
            compared_declaration("missing", None, None)

    def test_malformed_nested_identity_is_rejected(self) -> None:
        with self.assertRaisesRegex(TypeError, "expected bound declaration"):
            nested_declaration({"seed": {}}, "seed")


if __name__ == "__main__":
    unittest.main()
