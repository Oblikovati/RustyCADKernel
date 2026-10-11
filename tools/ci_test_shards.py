#!/usr/bin/env python3
"""Bound Martensite CI disk use without omitting workspace packages or doctests.

Example: python3 tools/ci_test_shards.py --shard 0 --count 6 --runner test
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import json
from pathlib import Path
import subprocess
from typing import Protocol

RUNNER_PREFIXES: dict[str, list[str]] = {
    "test": ["cargo", "test"],
    "nextest": ["cargo", "nextest", "run", "--profile", "ci"],
    "coverage": [
        "cargo",
        "llvm-cov",
        "--lcov",
        "--output-path",
        "target/lcov.info",
    ],
}


@dataclass(frozen=True)
class PackageWeight:
    """A workspace member's executable footprint estimate, e.g. PackageWeight('a', 5)."""

    name: str
    targets: int


class CargoProcess(Protocol):
    """Process boundary for real Cargo and deterministic test doubles."""

    def metadata(self, root: Path) -> list[PackageWeight]: ...
    def run(self, command: list[str], root: Path) -> None: ...


class NativeCargo:
    """Read Cargo's workspace and execute checked commands; NativeCargo()."""

    def metadata(self, root: Path) -> list[PackageWeight]:
        """Discover members, including newly added crates; NativeCargo().metadata(root)."""
        output = subprocess.check_output(
            ["cargo", "metadata", "--no-deps", "--locked", "--format-version", "1"],
            cwd=root,
            text=True,
        )
        return metadata_packages(json.loads(output))

    def run(self, command: list[str], root: Path) -> None:
        """Propagate failed tests to CI; NativeCargo().run(['cargo', 'test'], root)."""
        subprocess.run(command, cwd=root, check=True)


def metadata_packages(metadata: dict[str, object]) -> list[PackageWeight]:
    """Count every target of workspace members; metadata_packages(cargo_metadata)."""
    members = set(metadata["workspace_members"])
    return [
        PackageWeight(package["name"], max(1, len(package["targets"])))
        for package in metadata["packages"]
        if package["id"] in members
    ]


def shard_packages(
    packages: list[PackageWeight], count: int
) -> list[list[PackageWeight]]:
    """Balance target counts with stable name tie-breaking; shard_packages(packages, 6)."""
    if not 1 <= count <= len(packages):
        raise ValueError(f"Shard count {count}: expected 1..{len(packages)}")
    shards: list[list[PackageWeight]] = [[] for _ in range(count)]
    totals = [0] * count
    # Run 38075760896 exhausted disk by linking all workspace tests on one runner.
    for package in sorted(packages, key=lambda item: (-item.targets, item.name)):
        index = min(range(count), key=lambda index: (totals[index], index))
        shards[index].append(package)
        totals[index] += package.targets
    return shards


def shard_command(runner: str, packages: list[PackageWeight]) -> list[str]:
    """Keep native cargo test doctests and all features; shard_command('test', packages)."""
    if runner not in RUNNER_PREFIXES or not packages:
        raise ValueError(
            f"Runner {runner!r}, packages {packages!r}: expected known runner and nonempty shard"
        )
    return (
        RUNNER_PREFIXES[runner]
        + (["--no-fail-fast"] if runner == "test" else [])
        + ["--all-features", "--locked"]
        + [argument for package in packages for argument in ["--package", package.name]]
    )


def run_shard(
    root: Path, shard: int, count: int, runner: str, cargo: CargoProcess
) -> None:
    """Run one exhaustive partition; run_shard(root, 0, 6, 'test', NativeCargo())."""
    if not 0 <= shard < count:
        raise ValueError(f"Shard index {shard}: expected 0..{count - 1}")
    packages = shard_packages(cargo.metadata(root), count)[shard]
    print(
        f"Martensite shard {shard + 1}/{count}: " + ", ".join(p.name for p in packages),
        flush=True,
    )
    cargo.run(shard_command(runner, packages), root)


def main() -> None:
    """Run the requested CI partition; main()."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--shard", type=int, required=True)
    parser.add_argument("--count", type=int, required=True)
    parser.add_argument(
        "--runner", choices=["test", "nextest", "coverage"], required=True
    )
    args = parser.parse_args()
    run_shard(
        Path(__file__).resolve().parents[1],
        args.shard,
        args.count,
        args.runner,
        NativeCargo(),
    )


if __name__ == "__main__":
    main()
