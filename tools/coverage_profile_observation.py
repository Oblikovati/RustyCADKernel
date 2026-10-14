"""Read the real tool-selected collector through an owned command interface."""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
import json
from pathlib import Path
import shlex
from typing import Protocol

from coverage_profile_policy import (
    coverage_profile_directory,
    coverage_profile_environment,
)


class CoverageEnvironmentCommands(Protocol):
    """Read-only external boundary; named fakes return metadata and show-env exports."""

    def capture(self, root: Path, argv: list[str], environment: dict[str, str]) -> str:
        """Capture one actual tool response.

        Example: commands.capture(root, argv, environment).
        """
        ...


@dataclass(frozen=True)
class ObservedCoverageProfile:
    """Keep actual collector location, inherited configuration and raw evidence together."""

    profile_path: Path
    directory: Path
    environment: dict[str, str]
    show_env: str


def cargo_metadata_target(
    root: Path, environment: dict[str, str], commands: CoverageEnvironmentCommands
) -> Path:
    """Honor Cargo config as well as env overrides; cargo_metadata_target(root, env, commands)."""
    argv = ["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"]
    response = commands.capture(root, argv, environment)
    metadata = decode_coverage_target_metadata(response)
    target = metadata.get("target_directory")
    if not isinstance(target, str) or not Path(target).is_absolute():
        raise ValueError(
            f"received Cargo target_directory {target!r}; expected an absolute path string"
        )
    return Path(target)


def decode_coverage_target_metadata(response: str) -> dict[str, object]:
    """Preserve actual target metadata shape errors; decode_coverage_target_metadata(response)."""
    try:
        metadata = json.loads(response)
    except json.JSONDecodeError as error:
        raise ValueError(
            f"received Cargo metadata {response!r}; expected JSON object with target_directory"
        ) from error
    if not isinstance(metadata, dict):
        raise ValueError(
            f"received Cargo metadata {metadata!r}; expected JSON object with target_directory"
        )
    return metadata


def profile_path_from_show_env(export: str) -> Path:
    """Read LLVM_PROFILE_FILE without evaluating shell code; profile_path_from_show_env(export)."""
    for line in export.splitlines():
        if not line.startswith("export LLVM_PROFILE_FILE="):
            continue
        return coverage_export_path(line)
    raise ValueError(
        f"received show-env output {export!r}; expected export LLVM_PROFILE_FILE with an absolute path"
    )


def coverage_export_path(line: str) -> Path:
    """Parse one quoted assignment without shell evaluation; coverage_export_path(line)."""
    parts = split_coverage_export(line)
    value = parts[1].partition("=")[2]
    if value and Path(value).is_absolute():
        return Path(value)
    raise ValueError(
        f"received LLVM_PROFILE_FILE {value!r}; expected an absolute profile path"
    )


def split_coverage_export(line: str) -> list[str]:
    """Reject malformed tool output with its offending line; split_coverage_export(line)."""
    try:
        parts = shlex.split(line)
    except ValueError as error:
        raise ValueError(
            f"received show-env assignment {line!r}; expected a quoted LLVM_PROFILE_FILE assignment"
        ) from error
    if len(parts) != 2:
        raise ValueError(
            f"received show-env assignment {line!r}; expected one LLVM_PROFILE_FILE assignment"
        )
    return parts


def observe_compiler_profile(
    root: Path, environment: Mapping[str, str], commands: CoverageEnvironmentCommands
) -> ObservedCoverageProfile:
    """Capture the same collector used by normal coverage; observe_compiler_profile(root, env, commands)."""
    selected = coverage_profile_environment(environment)
    target = cargo_metadata_target(root, selected, commands)
    selected["CARGO_LLVM_COV_TARGET_DIR"] = str(
        coverage_profile_directory(root, selected, target)
    )
    export = commands.capture(root, ["cargo", "llvm-cov", "show-env", "--sh"], selected)
    profile = profile_path_from_show_env(export)
    validate_observed_profile_location(profile, selected)
    return ObservedCoverageProfile(profile, profile.parent, selected, export)


def validate_observed_profile_location(
    profile: Path, environment: Mapping[str, str]
) -> None:
    """Reject divergence between collector and child location; validate_observed_profile_location(profile, env)."""
    expected = (
        Path(environment["CARGO_LLVM_COV_TARGET_DIR"])
        / environment["LLVM_PROFILE_FILE_NAME"]
    )
    if profile != expected:
        raise ValueError(
            f"received tool-selected profile {profile!s}; expected actual collector {expected!s}"
        )
