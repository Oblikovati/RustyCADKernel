"""Mapping guards prevent empty, repeated, or collapsed typed coverage."""
import unittest
from tkernel_mapping import mapped_runtime_sets, runtime_pairs, listed_runtime_cases


class RuntimeMappingTests(unittest.TestCase):
    def test_typed_listing_comments_keep_each_runtime_identity(self):
        listing = "Plain.\n  Case\nTyped/0.  # TypeParam = int\n  Case\nTyped/1.  # TypeParam = long\n  Case\n"
        self.assertEqual(listed_runtime_cases(listing), {"Plain.Case", "Typed/0.Case", "Typed/1.Case"})
        with self.assertRaises(ValueError):
            listed_runtime_cases("Plain.\n  Case\n  Case\n")

    def test_regular_and_expanded_cases(self):
        plain = {"upstream_id": "Suite.Plain", "rust_target": "suite", "rust_test": "plain"}
        typed = {"upstream_id": "Typed.Case", "rust_target": "typed", "runtime_cases": [
            {"upstream_case": "Typed/0.Case", "rust_test": "int::case"},
            {"upstream_case": "Typed/1.Case", "rust_test": "long::case"}]}
        native, rust = mapped_runtime_sets([plain, typed])
        self.assertEqual(native, {"Suite.Plain", "Typed/0.Case", "Typed/1.Case"})
        self.assertEqual(rust, {("suite", "plain"), ("typed", "int::case"), ("typed", "long::case")})

    def test_empty_or_duplicate_expansions_fail(self):
        first = {"upstream_case": "Typed/0.Case", "rust_test": "case0"}
        for cases in [[], [first, first], [first, {"upstream_case": "Typed/1.Case", "rust_test": "case0"}]]:
            with self.subTest(cases=cases), self.assertRaises(ValueError):
                runtime_pairs({"upstream_id": "Typed.Case", "runtime_cases": cases})

    def test_reused_runtime_identity_across_declarations_fails(self):
        row = {"upstream_id": "Suite.Plain", "rust_target": "suite", "rust_test": "plain"}
        with self.assertRaises(ValueError):
            mapped_runtime_sets([row, row])


if __name__ == "__main__":
    unittest.main()
