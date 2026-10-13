#!/usr/bin/env python3
"""Build and test Martensite Cargo release assets before publication.

Example: python3 tools/release_packages.py --tag v0.1.0
Requires Python 3.12+, the pinned Rust toolchain, and native build prerequisites.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import gzip
from pathlib import Path
import tempfile
import tarfile

from release_failure_commands import ReleaseDiagnosticCommands
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
    for features in ([], ["--all-features"]):
        selected = (
            crates if not features else [crate for crate in crates if crate.features]
        )
        if not selected:
            continue
        for crate in selected:
            test_profile(stage, commands, features, crate.name)
        # Toolkit suites bound the footprint; preserve dependency builds across packages.
        commands.run(["cargo", "clean", "--target-dir", str(stage / "target")], stage)


def test_profile(
    stage: Path, commands: Commands, features: list[str], package: str
) -> None:
    """Test all targets and documentation for one feature profile; example: ``test_profile(stage, runner, [], 'rustycad-kernel')``."""
    for targets in (["--all-targets"], ["--doc"]):
        arguments = [
            "cargo",
            "test",
            "--package",
            package,
            "--locked",
            "--target-dir",
            str(stage / "target"),
            *features,
            *targets,
        ]
        print("+ " + " ".join(arguments), flush=True)
        print(commands.run(arguments, stage), end="", flush=True)


def build_release(root: Path, tag: str, commands: Commands) -> Path:
    """Produce tested assets only for a clean tagged revision; example: ``build_release(root, 'v0.1.0', runner)``."""
    crates, revision, output = release_plan(root, tag, commands)
    prepare_release_packages(root, commands)
    verify_extracted_release(root, tag, revision, commands, crates)
    seal_release(root / "target/package", output, crates, revision, root / "Cargo.lock")
    return output


def release_plan(
    root: Path, tag: str, commands: Commands
) -> tuple[list[ReleaseCrate], str, Path]:
    """Validate identity before creating receipts; e.g. release_plan(root, 'v0.2.3', runner)."""
    crates = release_inventory(root, commands)
    if tag != f"v{crates[0].version}":
        raise ValueError(f"release tag {tag!r}: expected v{crates[0].version}")
    revision = release_revision(root, commands, tag)
    output = root / "target/releases" / tag
    if output.exists():
        raise ValueError(f"release output {str(output)!r}: expected a new directory")
    return crates, revision, output


def prepare_release_packages(root: Path, commands: Commands) -> None:
    """Keep existing discovery/package ordering; e.g. prepare_release_packages(root, runner)."""
    # Run 38315068263 exposed a regression module missing from generated test groups.
    commands.run(["python3", "tools/toolkit_test_suites.py"], root)
    # Cargo cannot resolve unpublished registry siblings. Verify the whole archive
    # set below instead; retain the owner lockfile as a separate release asset.
    commands.run(
        ["cargo", "package", "--workspace", "--no-verify", "--exclude-lockfile"], root
    )
    package_test_support(root, commands)


def verify_extracted_release(
    root: Path, tag: str, revision: str, commands: Commands, crates: list[ReleaseCrate]
) -> None:
    """Keep cold failure receipts outside deleted stage; e.g. verify_extracted_release(root, tag, sha, runner, crates)."""
    with tempfile.TemporaryDirectory(prefix="martensite-packages-") as temporary:
        diagnostics = Path(
            tempfile.mkdtemp(prefix=f"{tag}-", dir=diagnostic_root(root))
        )
        logged = ReleaseDiagnosticCommands(commands, diagnostics / "commands")
        logged.write_json("release.json", {"tag": tag, "revision": revision})
        test_archives(root, Path(temporary) / "workspace", logged, crates)


def diagnostic_root(root: Path) -> Path:
    """Keep receipts outside removable stages; e.g. diagnostic_root(root)."""
    output = root / "target/release-diagnostics"
    output.mkdir(parents=True, exist_ok=True)
    return output


def package_test_support(root: Path, commands: Commands) -> None:
    """Bundle tracked oracle fixtures so archive tests need no checkout; example: ``package_test_support(root, runner)``."""
    tracked = commands.run(
        ["git", "ls-files", "-z", "tools/oracle", "porting/parity.json"], root
    ).split("\0")
    target = root / "target/package/martensite-test-support.tar.gz"
    with target.open("wb") as raw, gzip.GzipFile(
        filename="", fileobj=raw, mode="wb", mtime=0
    ) as compressed, tarfile.open(fileobj=compressed, mode="w") as stream:
        for name in sorted(filter(None, tracked)):
            stream.add(
                root / name, arcname=name, recursive=False, filter=release_metadata
            )


def release_metadata(member: tarfile.TarInfo) -> tarfile.TarInfo:
    """Remove checkout-specific archive metadata; example: ``release_metadata(member)``."""
    if not member.isfile():
        raise ValueError(
            f"test support member {member.name!r}: expected a regular tracked file"
        )
    # TarInfo holds its open archive; only scalar metadata needs replacement.
    return member.replace(
        uid=0, gid=0, uname="", gname="", mtime=0, mode=0o644, deep=False
    )


@dataclass(frozen=True)
class ReleaseArguments:
    """Separate immutable kernel checkout from the checkout owning orchestration."""

    tag: str
    root: Path


def parse_release_arguments(arguments: list[str] | None = None) -> ReleaseArguments:
    """Select the kernel checkout explicitly; e.g. parse_release_arguments(['--tag', 'v0.2.3', '--root', '../kernel'])."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", required=True)
    parser.add_argument(
        "--root", type=Path, default=Path(__file__).resolve().parents[1]
    )
    selected = parser.parse_args(arguments)
    return ReleaseArguments(tag=selected.tag, root=selected.root.resolve())


def main() -> None:
    """Run current tools against a tagged kernel; e.g. python3 tools/release_packages.py --tag v0.2.3 --root ../kernel."""
    arguments = parse_release_arguments()
    print(build_release(arguments.root, arguments.tag, SystemCommands()))


if __name__ == "__main__":
    main()
