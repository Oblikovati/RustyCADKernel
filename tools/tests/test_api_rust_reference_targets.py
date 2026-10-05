"""Reference receiver selection must preserve source pinning and exact paths."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from api_crosswalk import rust_candidates, rust_crates, rust_target_prefix
from api_rust_reference_targets import nominal_reference_owner
from api_inventory_json import JSONValue, object_value, array_value


class BorrowedTargetFixture:
    def __init__(self, target: str) -> None:
        fixture_path = (
            Path(__file__).parent / "fixtures/rust-api-inventory-cmath-asinh.json"
        )
        self.crate = object_value(json.loads(fixture_path.read_text()), "fixture crate")
        declarations = array_value(self.crate["declarations"], "declarations")
        self.entry = object_value(declarations[0], "source-pinned declaration")
        self.entry["path"] = target
        self.crate["declarations"] = [self.entry]

    def candidates(self, target: str) -> tuple[str, list[dict[str, JSONValue]]]:
        """Resolve against the named fixture; e.g. `fixture.candidates(target)`."""
        return rust_candidates(target, rust_crates({"crates": [self.crate]}))


class ReferenceTargetTests(unittest.TestCase):
    def test_shared_mutable_and_named_lifetime_receivers_select_the_nominal_crate(
        self,
    ) -> None:
        for owner in [
            "&rustycad_cmath::Value",
            "&mut rustycad_cmath::Value",
            "&'a rustycad_cmath::Value",
            "&'static mut rustycad_cmath::Value",
        ]:
            target = f"<{owner} as core::ops::arith::Add<T>>::add"
            self.assertEqual(rust_target_prefix(target), ("rustycad_cmath", True))
            self.assertEqual(
                BorrowedTargetFixture(target).candidates(target)[0], "unique"
            )

    def test_shared_receiver_does_not_match_owned_or_mutable_body(self) -> None:
        target = "<&rustycad_cmath::Value as core::ops::arith::Add<T>>::add"
        fixture = BorrowedTargetFixture(target)
        for other in [target.replace("<&", "<"), target.replace("<&", "<&mut ")]:
            self.assertEqual(fixture.candidates(other), ("missing", []))

    def test_malformed_reference_qualifiers_never_choose_a_crate(self) -> None:
        for owner in [
            "&&crate::Value",
            "& crate::Value",
            "&mut  crate::Value",
            "&'12 crate::Value",
            "&'a",
            "&crate",
        ]:
            self.assertEqual(nominal_reference_owner(owner), "")

    def test_borrowed_body_still_requires_pinned_source(self) -> None:
        target = "<&rustycad_cmath::Value as core::ops::arith::Add<T>>::add"
        fixture = BorrowedTargetFixture(target)
        fixture.entry["source_sha256"] = None
        self.assertEqual(fixture.candidates(target)[0], "unverifiable_source")

    def test_owned_ufcs_and_direct_paths_are_unchanged(self) -> None:
        self.assertEqual(nominal_reference_owner("crate::Value"), "crate::Value")
        self.assertEqual(rust_target_prefix("crate::Value::new"), ("crate", True))
        self.assertEqual(
            rust_target_prefix("<crate::Value as Trait>::method"), ("crate", True)
        )


if __name__ == "__main__":
    unittest.main()
