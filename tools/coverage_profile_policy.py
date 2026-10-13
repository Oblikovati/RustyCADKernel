"""One bounded LLVM profile policy for Martensite coverage tests and probe shards."""

from __future__ import annotations

from collections.abc import Mapping
from pathlib import Path
import re

DEFAULT_PROFILE_FILENAME = "martensite-%4m.profraw"


def coverage_pool_filename(pool_size: int) -> str:
    """Keep runtime locking within the documented pool range; coverage_pool_filename(4)."""
    if (
        not isinstance(pool_size, int)
        or isinstance(pool_size, bool)
        or not 1 <= pool_size <= 9
    ):
        raise ValueError(
            f"received pool size {pool_size!r}; expected an integer from 1 through 9"
        )
    return f"martensite-%{pool_size}m.profraw"


def validate_coverage_filename(filename: str) -> str:
    """Reject PID fanout and paths outside the collector; validate_coverage_filename('unit-%4m.profraw')."""
    if not re.fullmatch(r"[A-Za-z0-9_.-]+-%(?:[1-9])?m\.profraw", filename):
        raise ValueError(
            f"received profile filename {filename!r}; expected a filename ending -%m.profraw or -%1m through -%9m.profraw"
        )
    return filename


def coverage_profile_environment(
    environment: Mapping[str, str], pool_size: int | None = None
) -> dict[str, str]:
    """Copy caller settings and select bounded merging; coverage_profile_environment({}, 4)."""
    filename = environment.get("LLVM_PROFILE_FILE_NAME", DEFAULT_PROFILE_FILENAME)
    if pool_size is not None:
        filename = coverage_pool_filename(pool_size)
    return dict(
        environment, LLVM_PROFILE_FILE_NAME=validate_coverage_filename(filename)
    )


def coverage_profile_directory(
    root: Path, environment: Mapping[str, str], cargo_target_directory: Path
) -> Path:
    """Use actual Cargo metadata or its explicit coverage override; coverage_profile_directory(root, {}, Path('/cache'))."""
    override = environment.get("CARGO_LLVM_COV_TARGET_DIR")
    if override is not None:
        return root / override
    return cargo_target_directory / "llvm-cov-target"
