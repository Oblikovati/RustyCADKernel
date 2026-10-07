"""Mapped Rust test targets must hold exactly their mapped legacy tests."""
from __future__ import annotations

import json
import unittest
from pathlib import Path

from legacy_pending_dependency import pending_dependency_rows, implemented_classes
from legacy_test_targets import (TargetMismatch, built_executables, mapped_target_tests, target_mismatches,
                                 terse_test_names)

ROOT = Path(__file__).resolve().parents[2]


class FakeTestLister:
    """Returns fixed listings in place of building and running cargo test targets."""

    def __init__(self, listings: dict[tuple[str, str], set[str]]) -> None:
        self.listings = listings
        self.requested: list[tuple[str, str]] = []

    def list_tests(self, targets: list[tuple[str, str]]) -> dict[tuple[str, str], set[str]]:
        self.requested = list(targets)
        return {key: self.listings[key] for key in targets if key in self.listings}


def row(identity: str, target: str, test: str, status: str = "translated") -> dict:
    return {"upstream_id": identity, "rust_package": "pkg", "rust_target": target, "rust_test": test, "status": status}


class MappedTargetTests(unittest.TestCase):
    def test_runtime_expansions_and_pending_rows(self) -> None:
        typed = dict(row("T.Case", "typed", "unused"),
                     runtime_cases=[{"upstream_case": "T/0.Case", "rust_test": "a"}, {"upstream_case": "T/1.Case", "rust_test": "b"}])
        pending = {"upstream_id": "P.Case", "status": "pending_dependency"}
        mapped = mapped_target_tests([row("S.Case", "suite", "s_case"), typed, pending])
        self.assertEqual(mapped, {("pkg", "suite"): {"s_case"}, ("pkg", "typed"): {"a", "b"}})

    def test_a_target_with_port_only_tests_is_reported(self) -> None:
        # rustycad-tkgeomalgo/geom_fill_filling_behavior held one upstream test beside nine port-only ones.
        lister = FakeTestLister({("pkg", "mixed"): {"upstream", "port_only"}, ("pkg", "clean"): {"only"}})
        mismatches = target_mismatches({("pkg", "mixed"): {"upstream"}, ("pkg", "clean"): {"only"}}, lister)
        self.assertEqual(mismatches, [TargetMismatch("pkg", "mixed", ("port_only",), ())])
        self.assertEqual(lister.requested, [("pkg", "clean"), ("pkg", "mixed")])

    def test_a_missing_mapped_test_is_reported(self) -> None:
        mismatches = target_mismatches({("pkg", "t"): {"a", "b"}}, FakeTestLister({("pkg", "t"): {"a"}}))
        self.assertEqual(mismatches, [TargetMismatch("pkg", "t", (), ("b",))])

    def test_a_target_without_listing_is_an_error(self) -> None:
        with self.assertRaisesRegex(ValueError, "No test listing"):
            target_mismatches({("pkg", "t"): {"a"}}, FakeTestLister({}))


class CargoOutputParsingTests(unittest.TestCase):
    def test_terse_listing_keeps_tests_only(self) -> None:
        listing = "mod::case: test\nother: test\nbench_one: bench\n\n2 tests, 1 benchmark\n"
        self.assertEqual(terse_test_names(listing), {"mod::case", "other"})

    def test_built_executables_keep_test_artifacts_only(self) -> None:
        messages = [
            {"reason": "compiler-artifact", "target": {"name": "suite", "kind": ["test"]}, "executable": "/t/suite-1"},
            {"reason": "compiler-artifact", "target": {"name": "lib", "kind": ["lib"]}, "executable": None},
            {"reason": "build-finished", "success": True},
        ]
        text = "\n".join(map(json.dumps, messages)) + "\nCompiling note\n"
        self.assertEqual(built_executables(text), {"suite": Path("/t/suite-1")})


class LedgerPendingRowsTests(unittest.TestCase):
    def test_every_pending_row_of_the_ledger_is_valid_and_still_blocked(self) -> None:
        ledger = json.loads((ROOT / "porting/parity.json").read_text())
        pending = pending_dependency_rows(ledger["tests"], implemented_classes(ledger["api"]))
        self.assertTrue(all(case.blocked_by for case in pending.values()))


if __name__ == "__main__":
    unittest.main()
