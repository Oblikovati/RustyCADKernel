"""Coverage options may change profile storage, never the selected suite or failure exit."""

from pathlib import Path
import os
import subprocess
import unittest
from unittest import mock

from run_coverage import (
    COVERAGE_EXCLUSION,
    SystemCoverageCommands,
    coverage_parser,
    main,
    observe_coverage_policy,
    run_workspace_coverage,
    workspace_coverage_arguments,
)


class NamedCoverageCommands:
    """Record the exact invocation and return the owned fake's configured exit."""

    def __init__(self, exit_code: int = 0) -> None:
        self.exit_code = exit_code
        self.calls: list[tuple[Path, list[str], dict[str, str]]] = []
        self.captures: list[tuple[Path, list[str], dict[str, str]]] = []
        self.records: list[dict[str, str]] = []

    def execute(self, root: Path, argv: list[str], environment: dict[str, str]) -> int:
        """Keep the explicit observer state; see the enclosing named fixture."""

        self.calls.append((root, argv.copy(), environment.copy()))
        return self.exit_code

    def capture(self, root: Path, argv: list[str], environment: dict[str, str]) -> str:
        self.captures.append((root, argv.copy(), environment.copy()))
        if argv[:2] == ["cargo", "metadata"]:
            return '{"target_directory":"/configured/cache"}'
        return (
            "export LLVM_PROFILE_FILE='"
            + environment["CARGO_LLVM_COV_TARGET_DIR"]
            + "/"
            + environment["LLVM_PROFILE_FILE_NAME"]
            + "'\n"
        )

    def record(self, observation: dict[str, str]) -> None:
        """Keep the explicit observer state; see the enclosing named fixture."""

        self.records.append(observation.copy())


class NamedCoverageSubprocess:
    """Observe the system boundary without building or running a workspace."""

    def __init__(self) -> None:
        """Keep the explicit observer state; see the enclosing named fixture."""

        self.called = False

    def run(
        self, argv: list[str], *, cwd: Path, env: dict[str, str], check: bool
    ) -> subprocess.CompletedProcess[str]:
        self.called = (
            argv == ["cargo"]
            and cwd == Path("/kernel")
            and env == {"LANG": "C"}
            and not check
        )
        return subprocess.CompletedProcess(argv, 101)


class NamedCoverageCaptureSubprocess:
    """Return an actual-looking export while recording the capture boundary."""

    def __init__(self) -> None:
        """Keep the explicit observer state; see the enclosing named fixture."""

        self.called = False

    def run(
        self,
        argv: list[str],
        *,
        cwd: Path,
        env: dict[str, str],
        text: bool,
        capture_output: bool,
        check: bool
    ) -> subprocess.CompletedProcess[str]:
        self.called = (
            text and capture_output and check and cwd == Path("/kernel") and env == {}
        )
        return subprocess.CompletedProcess(
            argv, 0, "export LLVM_PROFILE_FILE='pool-%4m.profraw'\n"
        )


class CoverageRunnerTests(unittest.TestCase):
    def test_workspace_command_is_exact_ci_suite_profile_and_existing_exclusion(
        self,
    ) -> None:
        expected = "cargo llvm-cov nextest --workspace --all-features --locked --profile ci".split()
        expected += [
            "-E",
            COVERAGE_EXCLUSION,
            "--lcov",
            "--output-path",
            "target/lcov.info",
        ]
        self.assertEqual(
            workspace_coverage_arguments(Path("target/lcov.info")), expected
        )
        self.assertEqual(
            COVERAGE_EXCLUSION,
            "not test(=bvh_bug30655::BVH_PairDistanceTest__Bug30655_SphereDistance) & not test(=sketch_region_prism::owned_request::complete::cancellation_tests::every_complete_checkpoint_cancel_and_native_failure_is_terminal_and_reusable)",
        )

    def test_environment_options_preserve_selection_context_and_failure_exit(
        self,
    ) -> None:
        original = {
            "LANG": "C",
            "VK_DRIVER_FILES": "pinned.json",
            "CARGO_LLVM_COV_TARGET_DIR": "scratch",
            "LLVM_PROFILE_FILE_NAME": "custom-%2m.profraw",
        }
        fake = NamedCoverageCommands(101)
        self.assertEqual(
            run_workspace_coverage(Path("/kernel"), Path("out.info"), original, fake),
            101,
        )
        self.assertEqual(
            fake.calls[0][2],
            original | {"CARGO_LLVM_COV_TARGET_DIR": "/kernel/scratch"},
        )
        self.assertEqual(original["LLVM_PROFILE_FILE_NAME"], "custom-%2m.profraw")

    def test_explicit_pool_changes_only_profile_filename_not_selected_command(
        self,
    ) -> None:
        original = {"LLVM_PROFILE_FILE_NAME": "custom-%2m.profraw"}
        fake = NamedCoverageCommands()
        run_workspace_coverage(Path("/kernel"), Path("out.info"), original, fake)
        run_workspace_coverage(Path("/kernel"), Path("out.info"), original, fake, 4)
        self.assertEqual(fake.calls[1][1], fake.calls[0][1])
        self.assertEqual(
            fake.calls[1][2]["LLVM_PROFILE_FILE_NAME"], "martensite-%4m.profraw"
        )

    def test_invalid_policy_is_rejected_before_any_external_command(self) -> None:
        fake = NamedCoverageCommands()
        with self.assertRaisesRegex(ValueError, "received pool size 32"):
            run_workspace_coverage(Path("/kernel"), Path("out.info"), {}, fake, 32)
        self.assertEqual(fake.calls, [])

    def test_system_boundary_preserves_original_exit_and_has_no_shell(self) -> None:
        fake = NamedCoverageSubprocess()
        with mock.patch("run_coverage.subprocess.run", fake.run):
            self.assertEqual(
                SystemCoverageCommands().execute(
                    Path("/kernel"), ["cargo"], {"LANG": "C"}
                ),
                101,
            )
        self.assertTrue(fake.called)

    def test_actual_tool_export_and_selected_policy_are_recorded_without_mutation(
        self,
    ) -> None:
        fake = NamedCoverageCommands()
        environment = {
            "LLVM_PROFILE_FILE_NAME": "custom-%4m.profraw",
            "CARGO_TARGET_DIR": "scratch",
        }
        observe_coverage_policy(Path("/kernel"), environment, fake)
        self.assertEqual(fake.captures[1][1], ["cargo", "llvm-cov", "show-env", "--sh"])
        self.assertEqual(
            fake.captures[1][2]["CARGO_LLVM_COV_TARGET_DIR"],
            "/configured/cache/llvm-cov-target",
        )
        self.assertNotIn("CARGO_LLVM_COV_TARGET_DIR", environment)
        self.assertEqual(fake.records[0]["selected_filename"], "custom-%4m.profraw")
        self.assertIn(
            "/configured/cache/llvm-cov-target/custom-%4m.profraw",
            fake.records[0]["cargo_llvm_cov_show_env"],
        )

    def test_system_capture_and_structured_record_are_exercised(self) -> None:
        fake = NamedCoverageCaptureSubprocess()
        with mock.patch("run_coverage.subprocess.run", fake.run):
            result = SystemCoverageCommands().capture(Path("/kernel"), ["cargo"], {})
        self.assertTrue(fake.called)
        self.assertIn("pool-%4m", result)
        with mock.patch("sys.stdout") as output:
            SystemCoverageCommands().record({"event": "test"})
        self.assertTrue(output.write.called)

    def test_cli_defaults_and_explicit_options_are_owned(self) -> None:
        parser = coverage_parser()
        options = parser.parse_args([])
        self.assertEqual(options.output_path, Path("target/lcov.info"))
        self.assertIsNone(options.profile_pool_size)
        options = parser.parse_args(
            ["--output-path", "out.info", "--profile-pool-size", "2"]
        )
        self.assertEqual(
            (options.output_path, options.profile_pool_size), (Path("out.info"), 2)
        )

    def test_cli_passes_options_and_preserves_environment_without_running_coverage(
        self,
    ) -> None:
        fake = NamedCoverageCommands(3)
        before = os.environ.copy()
        with mock.patch("run_coverage.SystemCoverageCommands", return_value=fake):
            self.assertEqual(
                main(["--profile-pool-size", "4", "--output-path", "out.info"]), 3
            )
        self.assertEqual(
            fake.calls[0][2]["LLVM_PROFILE_FILE_NAME"], "martensite-%4m.profraw"
        )
        self.assertEqual(
            fake.calls[0][1], workspace_coverage_arguments(Path("out.info"))
        )
        self.assertEqual(dict(os.environ), before)

    def test_cli_invalid_configuration_exits_two_before_running(self) -> None:
        fake = NamedCoverageCommands()
        with mock.patch(
            "run_coverage.SystemCoverageCommands", return_value=fake
        ), mock.patch("sys.stderr"):
            with self.assertRaises(SystemExit) as caught:
                main(["--profile-pool-size", "0"])
        self.assertEqual(caught.exception.code, 2)
        self.assertEqual(fake.calls, [])


if __name__ == "__main__":
    unittest.main()
