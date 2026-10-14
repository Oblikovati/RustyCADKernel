"""Bounded files retain binary separation and caller-owned configuration."""

from pathlib import Path
import unittest

from coverage_profile_policy import (
    coverage_pool_filename,
    coverage_profile_directory,
    coverage_profile_environment,
    validate_coverage_filename,
)


class CoverageProfilePolicyTests(unittest.TestCase):
    def test_all_documented_pool_sizes_generate_valid_bounded_patterns(self) -> None:
        for size in range(1, 10):
            self.assertEqual(
                validate_coverage_filename(coverage_pool_filename(size)),
                f"martensite-%{size}m.profraw",
            )
        self.assertEqual(
            validate_coverage_filename("custom-%m.profraw"), "custom-%m.profraw"
        )

    def test_invalid_pool_sizes_include_the_actual_value_and_expected_range(
        self,
    ) -> None:
        for size in [-1, 0, 10, 32, True]:
            with self.assertRaisesRegex(
                ValueError, "expected an integer from 1 through 9"
            ) as caught:
                coverage_pool_filename(size)
            self.assertIn(repr(size), str(caught.exception))

    def test_pid_paths_missing_pool_and_unsupported_pools_are_rejected(self) -> None:
        for filename in [
            "unit-%p-%4m.profraw",
            "unit-%32m.profraw",
            "unit-%0m.profraw",
            "unit.profraw",
            "../unit-%4m.profraw",
            "/unit-%4m.profraw",
            "unit-%4m-%4m.profraw",
            "unit-%4m.profdata",
        ]:
            with self.assertRaisesRegex(ValueError, "expected a filename") as caught:
                validate_coverage_filename(filename)
            self.assertIn(repr(filename), str(caught.exception))

    def test_environment_copy_honors_override_without_global_or_caller_mutation(
        self,
    ) -> None:
        original = {"LANG": "C", "LLVM_PROFILE_FILE_NAME": "custom-%2m.profraw"}
        result = coverage_profile_environment(original)
        self.assertEqual(result, original)
        self.assertIsNot(result, original)
        self.assertEqual(
            coverage_profile_environment(original, 4)["LLVM_PROFILE_FILE_NAME"],
            "martensite-%4m.profraw",
        )
        self.assertEqual(original["LLVM_PROFILE_FILE_NAME"], "custom-%2m.profraw")
        self.assertEqual(
            coverage_profile_environment({})["LLVM_PROFILE_FILE_NAME"],
            "martensite-%4m.profraw",
        )

    def test_metadata_target_preserves_normal_and_environment_selected_directory(
        self,
    ) -> None:
        root = Path("/kernel")
        self.assertEqual(
            coverage_profile_directory(root, {}, root / "target"),
            root / "target/llvm-cov-target",
        )
        self.assertEqual(
            coverage_profile_directory(
                root, {"CARGO_TARGET_DIR": "compact"}, root / "compact"
            ),
            root / "compact/llvm-cov-target",
        )

    def test_explicit_coverage_target_overrides_the_actual_cargo_target(self) -> None:
        root = Path("/kernel")
        self.assertEqual(
            coverage_profile_directory(
                root, {"CARGO_LLVM_COV_TARGET_DIR": "/scratch"}, root / "target"
            ),
            Path("/scratch"),
        )
        environment = {
            "CARGO_TARGET_DIR": "ordinary",
            "CARGO_LLVM_COV_TARGET_DIR": "instrumented",
        }
        self.assertEqual(
            coverage_profile_directory(root, environment, root / "ordinary"),
            root / "instrumented",
        )

    def test_config_target_directory_comes_from_actual_metadata(self) -> None:
        self.assertEqual(
            coverage_profile_directory(Path("/kernel"), {}, Path("/configured/cache")),
            Path("/configured/cache/llvm-cov-target"),
        )


if __name__ == "__main__":
    unittest.main()
