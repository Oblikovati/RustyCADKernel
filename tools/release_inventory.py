"""Validate the Martensite package contract consumed by Oblikovati.

Example: ``release_inventory(Path('.'), SystemCommands())``.
"""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
import json
from pathlib import Path
import re
import subprocess
import sys
from typing import Protocol


class Commands(Protocol):
    """Owned command boundary; tests supply recorded command results."""

    def run(self, arguments: list[str], root: Path) -> str:
        """Execute a checked command, for example ``run(['git', 'rev-parse', 'HEAD'], root)``."""
        ...


class SystemCommands:
    """Execute commands without a shell and preserve their failure diagnostics."""

    last_stderr: str = ""

    def run(self, arguments: list[str], root: Path) -> str:
        """Return stdout, for example ``run(['cargo', '--version'], root)``."""
        result = subprocess.run(
            arguments,
            cwd=root,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        self.last_stderr = result.stderr
        print(result.stderr, end="", file=sys.stderr, flush=True)
        if result.returncode:
            print(result.stdout, end="", flush=True)
            result.check_returncode()
        return result.stdout


@dataclass(frozen=True)
class ReleaseCrate:
    """One owner-versioned package and its expected Cargo archive name."""

    name: str
    version: str
    directory: str
    features: tuple[str, ...] = ()

    @property
    def archive(self) -> str:
        """Cargo archive filename; for example ``ReleaseCrate('rustycad-kernel', '0.1.0', 'crates/kernel').archive``."""
        return f"{self.name}-{self.version}.crate"


def release_inventory(root: Path, commands: Commands) -> list[ReleaseCrate]:
    """Discover and validate every workspace crate; example: ``release_inventory(root, runner)``."""
    output = commands.run(
        ["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"], root
    )
    metadata = json.loads(output)
    members = set(metadata["workspace_members"])
    packages = [package for package in metadata["packages"] if package["id"] in members]
    crates = [release_crate_from_metadata(package, root) for package in packages]
    validate_inventory(crates)
    return sorted(crates, key=lambda crate: crate.name)


def release_crate_from_metadata(
    package: Mapping[str, object], root: Path
) -> ReleaseCrate:
    """Validate one Cargo package; e.g. release_crate_from_metadata(metadata_package, root)."""
    for field in ("name", "version", "manifest_path"):
        if not isinstance(package.get(field), str):
            raise ValueError(f"Cargo {field}={package.get(field)!r}; expected a string")
    return ReleaseCrate(
        str(package["name"]),
        str(package["version"]),
        package_directory(str(package["manifest_path"]), root),
        release_feature_names(package),
    )


def release_feature_names(package: Mapping[str, object]) -> tuple[str, ...]:
    """Require actual Cargo feature metadata; e.g. release_feature_names({'features': {'debug': []}})."""
    feature_map = package.get("features")
    if not isinstance(feature_map, dict):
        raise ValueError(f"Cargo features={feature_map!r}; expected a feature-name map")
    for name, enabled in feature_map.items():
        if not isinstance(name, str) or not isinstance(enabled, list):
            raise ValueError(
                f"Cargo feature={name!r}:{enabled!r}; expected a name and string list"
            )
        if any(not isinstance(feature, str) for feature in enabled):
            raise ValueError(
                f"Cargo feature={name!r}:{enabled!r}; expected a name and string list"
            )
    return tuple(sorted(feature_map))


def package_directory(manifest: str, root: Path) -> str:
    """Normalize package paths for portable release receipts; example: ``package_directory(path, root)``."""
    directory = Path(manifest).parent.relative_to(root)
    return directory.as_posix()


def validate_inventory(crates: list[ReleaseCrate]) -> None:
    """Require one stable SemVer and unique owned names; example: ``validate_inventory(crates)``."""
    versions = {crate.version for crate in crates}
    if len(versions) != 1 or not re.fullmatch(
        r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)", next(iter(versions), "")
    ):
        raise ValueError(
            f"package versions {sorted(versions)!r}: expected one stable major.minor.patch version"
        )
    for crate in crates:
        if not re.fullmatch(r"crates/[a-z0-9_-]+", crate.directory):
            raise ValueError(
                f"crate directory {crate.directory!r}: expected crates/NAME"
            )
    validate_names(crates)


def validate_names(crates: list[ReleaseCrate]) -> None:
    """Require unique owned package names; example: ``validate_names(crates)``."""
    names = [crate.name for crate in crates]
    if len(set(names)) != len(names) or any(
        not re.fullmatch(r"(rustycad|martensite)-[a-z0-9-]+", name) for name in names
    ):
        raise ValueError(
            f"package names {names!r}: expected unique rustycad-* or martensite-* names"
        )


def release_revision(root: Path, commands: Commands, tag: str) -> str:
    """Require a clean checkout at its immutable release tag; example: ``release_revision(root, runner, 'v0.1.0')``."""
    revision = commands.run(["git", "rev-parse", "HEAD"], root).strip()
    if not re.fullmatch("[0-9a-f]{40}", revision):
        raise ValueError(f"revision {revision!r}: expected a full Git commit SHA")
    dirty = commands.run(["git", "status", "--porcelain"], root).strip()
    if dirty:
        raise ValueError(f"working tree {dirty!r}: expected a clean release checkout")
    tagged = commands.run(["git", "rev-parse", f"{tag}^{{commit}}"], root).strip()
    if tagged != revision:
        raise ValueError(
            f"tag {tag!r} resolves to {tagged!r}: expected HEAD {revision}"
        )
    return revision
