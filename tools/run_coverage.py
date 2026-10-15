#!/usr/bin/env python3
"""Run unchanged Martensite workspace coverage with bounded per-binary profiles.

Example: python3 tools/run_coverage.py --output-path target/lcov.info
"""

from __future__ import annotations

import argparse
import json
from collections.abc import Mapping, Sequence
from pathlib import Path
import os
import subprocess
from typing import Protocol

from coverage_profile_policy import coverage_profile_environment
from coverage_profile_observation import observe_compiler_profile

COVERAGE_EXCLUSION = (
    "not test(=bvh_bug30655::BVH_PairDistanceTest__Bug30655_SphereDistance) & not test(=sketch_region_prism::owned_request::complete::cancellation_tests::every_complete_checkpoint_cancel_and_native_failure_is_terminal_and_reusable)"
)


class CoverageCommands(Protocol):
    """Owned external command boundary; named test fakes implement execute."""

    def execute(self, root: Path, argv: list[str], environment: dict[str, str]) -> int:
        """Return the original process exit.

        Example: commands.execute(root, argv, environment).
        """
        ...

    def capture(self, root: Path, argv: list[str], environment: dict[str, str]) -> str:
        """Capture tool-owned environment observation.

        Example: commands.capture(root, argv, environment).
        """
        ...

    def record(self, observation: dict[str, str]) -> None:
        """Retain the effective policy in structured output.

        Example: commands.record(observation).
        """
        ...


class SystemCoverageCommands:
    """Preserve nextest per-test processes and inherited process output."""

    def execute(self, root: Path, argv: list[str], environment: dict[str, str]) -> int:
        """Run directly without shell reinterpretation; commands.execute(root, argv, environment)."""
        result = subprocess.run(argv, cwd=root, env=environment, check=False)
        return result.returncode

    def capture(self, root: Path, argv: list[str], environment: dict[str, str]) -> str:
        """Keep actual cargo-llvm-cov output, not an inferred pattern; commands.capture(root, argv, environment)."""
        result = subprocess.run(
            argv, cwd=root, env=environment, text=True, capture_output=True, check=True
        )
        return result.stdout

    def record(self, observation: dict[str, str]) -> None:
        """Let the caller's gate log preserve the observation.

        Example: commands.record(observation).
        """
        print(json.dumps(observation, sort_keys=True), flush=True)


def workspace_coverage_arguments(output_path: Path) -> list[str]:
    """Preserve every CI selection and instrumentation option; workspace_coverage_arguments(Path('target/lcov.info'))."""
    return [
        "cargo",
        "llvm-cov",
        "nextest",
        "--workspace",
        "--all-features",
        "--locked",
        "--profile",
        "ci",
        "-E",
        COVERAGE_EXCLUSION,
        "--lcov",
        "--output-path",
        str(output_path),
    ]


def run_workspace_coverage(
    root: Path,
    output_path: Path,
    environment: Mapping[str, str],
    commands: CoverageCommands,
    pool_size: int | None = None,
) -> int:
    """Change profile storage only; run_workspace_coverage(root, output, env, commands)."""
    selected = coverage_profile_environment(environment, pool_size)
    observed = observe_coverage_policy(root, selected, commands)
    argv = workspace_coverage_arguments(output_path)
    return commands.execute(root, argv, observed)


def observe_coverage_policy(
    root: Path, environment: dict[str, str], commands: CoverageCommands
) -> dict[str, str]:
    """Record actual collector metadata before execution; observe_coverage_policy(root, env, commands)."""
    observed = observe_compiler_profile(root, environment, commands)
    commands.record(
        {
            "event": "martensite_coverage_profile_policy",
            "selected_filename": observed.environment["LLVM_PROFILE_FILE_NAME"],
            "actual_profile_path": str(observed.profile_path),
            "cargo_llvm_cov_show_env": observed.show_env,
        }
    )
    return observed.environment


def coverage_parser() -> argparse.ArgumentParser:
    """Expose local paths and the supported pool size; coverage_parser().parse_args([])."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-path", type=Path, default=Path("target/lcov.info"))
    parser.add_argument("--profile-pool-size", type=int)
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    """Keep invalid configuration out of execution; main(['--profile-pool-size', '4'])."""
    parser = coverage_parser()
    options = parser.parse_args(argv)
    root = Path(__file__).resolve().parents[1]
    try:
        return run_workspace_coverage(
            root,
            options.output_path,
            os.environ,
            SystemCoverageCommands(),
            options.profile_pool_size,
        )
    except ValueError as error:
        parser.error(str(error))
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
