"""Cargo config, tool exports and failures determine the actual coverage collector."""

from pathlib import Path
import json
import unittest

from coverage_profile_observation import (
    cargo_metadata_target,
    observe_compiler_profile,
    profile_path_from_show_env,
)


class NamedProfileEnvironmentCommands:
    """Emulate actual Cargo target metadata and cargo-llvm-cov's resulting export."""

    def __init__(self, target: str = "/configured/cache") -> None:
        self.target = target
        self.export_override: str | None = None
        self.metadata_override: str | None = None
        self.calls: list[tuple[list[str], dict[str, str]]] = []

    def capture(self, root: Path, argv: list[str], environment: dict[str, str]) -> str:
        self.calls.append((argv.copy(), environment.copy()))
        if argv[:2] == ["cargo", "metadata"]:
            return self.metadata_override or json.dumps(
                {"target_directory": self.target}
            )
        if self.export_override is not None:
            return self.export_override
        path = (
            environment["CARGO_LLVM_COV_TARGET_DIR"]
            + "/"
            + environment["LLVM_PROFILE_FILE_NAME"]
        )
        return "export LLVM_PROFILE_FILE='" + path + "'\n"


class CoverageProfileObservationTests(unittest.TestCase):
    def test_config_target_and_show_env_are_actual_injected_values(self) -> None:
        fake = NamedProfileEnvironmentCommands("/configured/cache with spaces")
        original = {"LLVM_PROFILE_FILE_NAME": "custom-%4m.profraw"}
        result = observe_compiler_profile(Path("/kernel"), original, fake)
        self.assertEqual(
            result.directory, Path("/configured/cache with spaces/llvm-cov-target")
        )
        self.assertEqual(result.profile_path, result.directory / "custom-%4m.profraw")
        self.assertEqual(
            result.environment["CARGO_LLVM_COV_TARGET_DIR"], str(result.directory)
        )
        self.assertIn("cache with spaces", result.show_env)
        self.assertEqual(original, {"LLVM_PROFILE_FILE_NAME": "custom-%4m.profraw"})

    def test_explicit_target_overrides_metadata_without_losing_actual_export(
        self,
    ) -> None:
        fake = NamedProfileEnvironmentCommands()
        result = observe_compiler_profile(
            Path("/kernel"), {"CARGO_LLVM_COV_TARGET_DIR": "coverage"}, fake
        )
        self.assertEqual(result.directory, Path("/kernel/coverage"))
        self.assertEqual(result.profile_path.name, "martensite-%4m.profraw")

    def test_actual_tool_filename_mismatch_is_an_error_not_an_inferred_success(
        self,
    ) -> None:
        fake = NamedProfileEnvironmentCommands()
        fake.export_override = "export LLVM_PROFILE_FILE='/unexpected/pid-%p.profraw'\n"
        with self.assertRaisesRegex(
            ValueError, "pid-%p.profraw.*expected actual collector"
        ):
            observe_compiler_profile(Path("/kernel"), {}, fake)

    def test_actual_tool_directory_mismatch_is_rejected_before_execution(self) -> None:
        fake = NamedProfileEnvironmentCommands()
        fake.export_override = (
            "export LLVM_PROFILE_FILE='/unexpected/martensite-%4m.profraw'\n"
        )
        with self.assertRaisesRegex(
            ValueError, "unexpected.*expected actual collector"
        ):
            observe_compiler_profile(Path("/kernel"), {}, fake)

    def test_missing_relative_and_wrong_shape_metadata_name_the_offending_value(
        self,
    ) -> None:
        for response in ["{}", '{"target_directory":"relative"}', "[]", "not json"]:
            fake = NamedProfileEnvironmentCommands()
            fake.metadata_override = response
            with self.assertRaisesRegex(ValueError, "received.*expected"):
                cargo_metadata_target(Path("/kernel"), {}, fake)

    def test_profile_export_parser_handles_spaces_and_rejects_bad_shapes(self) -> None:
        self.assertEqual(
            profile_path_from_show_env(
                "export OTHER=1\nexport LLVM_PROFILE_FILE='/cache with spaces/unit-%4m.profraw'\n"
            ),
            Path("/cache with spaces/unit-%4m.profraw"),
        )
        for response in [
            "",
            "export LLVM_PROFILE_FILE='relative.profraw'",
            "export LLVM_PROFILE_FILE='/unterminated",
            "export LLVM_PROFILE_FILE='/valid' extra",
        ]:
            with self.assertRaisesRegex(ValueError, "received.*expected"):
                profile_path_from_show_env(response)


if __name__ == "__main__":
    unittest.main()
