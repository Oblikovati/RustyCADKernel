"""Exact-identity, access-boundary and non-credit controls for effective obligations."""

from __future__ import annotations

import copy
import unittest

from api_inventory.effective_obligations import (
    effective_obligation_union,
    exact_declaration_index,
)
from api_inventory.inputs import JSONObject
from api_inventory.merge import declaration_identity


def native_row(name: str, public: bool, kind: str = "CXXMethod") -> JSONObject:
    row: JSONObject = {
        "kind": kind,
        "usr": f"c:@{name}",
        "signature": "void ()",
        "identity_signature": "void ()",
        "template_signature": "",
        "qualified_name": name,
        "access": "public" if public else "private",
        "api_visible": public,
        "exclusion_reason": "" if public else "private_ancestor",
    }
    row["id"] = declaration_identity(row)
    return row


class FakeNativeAccess:
    """Represent one protected producer, private record and inferred public member."""

    def __init__(self) -> None:
        self.producer = native_row("Owner::producer", True)
        self.record = native_row("Owner::Hidden", False, "CXXRecord")
        self.member = native_row("Owner::Hidden::call", False)
        self.member["access"] = "public"
        self.private = native_row("Owner::Hidden::secret", False)
        self.rows = [self.producer, self.record, self.member, self.private]
        self.edge = self.returned_edge()

    def returned_edge(self) -> JSONObject:
        return {
            "id": "edge",
            "status": "private_record_members_recorded",
            "producer_declaration": self.producer,
            "seed_declaration": self.producer,
            "target_declaration": self.record,
            "members": [
                {
                    "declaration": self.member,
                    "access_basis": "inferred_type_public_member",
                },
                {
                    "declaration": self.private,
                    "access_basis": "private_member_inaccessible",
                },
            ],
        }

    def contract(self, declaration: JSONObject, role: str) -> JSONObject:
        return {
            "native_id": declaration["id"],
            "qualified_name": declaration["qualified_name"],
            "signature": declaration["signature"],
            "kind": declaration["kind"],
            "role": role,
            "demonstrated_operations": ["fixture expression"],
            "private_name_accessible": False,
            "implementation_credit": False,
        }


class EffectiveObligationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.fixture = FakeNativeAccess()

    def test_member_and_record_are_separate_obligations(self) -> None:
        f = self.fixture
        contracts = [
            f.contract(f.member, "inferred_public_member"),
            f.contract(f.record, "inferred_record"),
        ]
        rows = effective_obligation_union(f.rows, [f.edge, f.edge], contracts)
        self.assertEqual(len(rows), 3)
        supplemental = next(row for row in rows if row["native_id"] == f.record["id"])
        self.assertFalse(supplemental["native_facts"]["api_visible"])
        self.assertEqual(len(supplemental["exposure_routes"]), 1)
        self.assertFalse(supplemental["implementation_credit"])

    def test_inference_alone_does_not_promote_obligation(self) -> None:
        f = self.fixture
        rows = effective_obligation_union(f.rows, [f.edge], [])
        self.assertEqual([row["native_id"] for row in rows], [f.producer["id"]])

    def test_private_member_and_unsupported_template_remain_excluded(self) -> None:
        f = self.fixture
        for edge, declaration in [
            (f.edge, f.private),
            ({**f.edge, "status": "unsupported_template_or_dependent_type"}, f.member),
        ]:
            with (
                self.subTest(declaration=declaration["qualified_name"]),
                self.assertRaisesRegex(ValueError, "eligible exact"),
            ):
                effective_obligation_union(
                    f.rows,
                    [edge],
                    [f.contract(declaration, "inferred_public_member")],
                )

    def test_reject_duplicate_or_fabricated_native_ids(self) -> None:
        f = self.fixture
        for rows in [f.rows + [f.member], [{**f.member, "id": "fabricated"}]]:
            with self.assertRaisesRegex(ValueError, "unique exact"):
                exact_declaration_index(rows)

    def test_reject_duplicate_proof_and_mismatched_signature(self) -> None:
        f = self.fixture
        contract = f.contract(f.member, "inferred_public_member")
        for proofs in [
            [contract, contract],
            [{**contract, "signature": "different overload"}],
        ]:
            with self.assertRaises(ValueError):
                effective_obligation_union(f.rows, [f.edge], proofs)

    def test_reject_private_name_or_credit_promotion(self) -> None:
        f = self.fixture
        for field in ["private_name_accessible", "implementation_credit"]:
            proof = {**f.contract(f.member, "inferred_public_member"), field: True}
            with self.assertRaises(ValueError):
                effective_obligation_union(f.rows, [f.edge], [proof])

    def test_reject_fabricated_embedded_producer(self) -> None:
        f = self.fixture
        edge = copy.deepcopy(f.edge)
        edge["producer_declaration"]["id"] = "wrong"
        with self.assertRaisesRegex(ValueError, "recomputed identity"):
            effective_obligation_union(f.rows, [edge], [])

    def test_reject_private_member_relabelled_in_edge(self) -> None:
        f = self.fixture
        f.member["access"] = "private"
        proof = f.contract(f.member, "inferred_public_member")
        with self.assertRaisesRegex(ValueError, "public member access"):
            effective_obligation_union(f.rows, [f.edge], [proof])


if __name__ == "__main__":
    unittest.main()
