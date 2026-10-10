#!/usr/bin/env python3
"""Build and test Martensite Cargo release assets before publication.

Example: python3 tools/release_packages.py --tag v0.1.0
Requires Python 3.12+, the pinned Rust toolchain, and native build prerequisites.
"""

from __future__ import annotations

import argparse
from pathlib import Path
import tempfile
import tarfile

from release_archives import seal_release, stage_workspace
from release_inventory import (
    Commands,
    ReleaseCrate,
    SystemCommands,
    release_inventory,
    release_revision,
)


def test_archives(
    root: Path, stage: Path, commands: Commands, crates: list[ReleaseCrate]
) -> None:
    """Compile and test extracted packages without original source paths; example: ``test_archives(root, stage, runner, crates)``."""
    stage_workspace(root / "target/package", stage, crates, root / "Cargo.lock")
    for targets in (["--all-targets"], ["--doc"]):
        output = commands.run(
            ["cargo", "test", "--workspace", "--all-features", "--locked", *targets],
            stage,
        )
        print(output, end="", flush=True)


def build_release(root: Path, tag: str, commands: Commands) -> Path:
    """Produce tested assets only for a clean tagged revision; example: ``build_release(root, 'v0.1.0', runner)``."""
    crates = release_inventory(root, commands)
    if tag != f"v{crates[0].version}":
        raise ValueError(f"release tag {tag!r}: expected v{crates[0].version}")
    revision = release_revision(root, commands, tag)
    output = root / "target/releases" / tag
    if output.exists():
        raise ValueError(f"release output {str(output)!r}: expected a new directory")
    # Cargo cannot resolve unpublished registry siblings. Verify the whole archive
    # set below instead; retain the owner lockfile as a separate release asset.
    commands.run(
        ["cargo", "package", "--workspace", "--no-verify", "--exclude-lockfile"], root
    )
    package_test_support(root, commands)
    with tempfile.TemporaryDirectory(prefix="martensite-packages-") as temporary:
        test_archives(root, Path(temporary) / "workspace", commands, crates)
    seal_release(root / "target/package", output, crates, revision, root / "Cargo.lock")
    return output


def package_test_support(root: Path, commands: Commands) -> None:
    """Bundle tracked oracle fixtures so archive tests need no checkout; example: ``package_test_support(root, runner)``."""
    tracked = commands.run(
        ["git", "ls-files", "-z", "tools/oracle", "porting/parity.json"], root
    ).split("\0")
    with tarfile.open(
        root / "target/package/martensite-test-support.tar.gz", "w:gz"
    ) as stream:
        for name in sorted(filter(None, tracked)):
            stream.add(root / name, arcname=name, recursive=False)


def main() -> None:
    """Run the tagged owner-side release check; example: ``python3 tools/release_packages.py --tag v0.1.0``."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", required=True)
    arguments = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    print(build_release(root, arguments.tag, SystemCommands()))


if __name__ == "__main__":
    main()
