"""Extract Cargo packages into an isolated, locked test workspace.

The kernel is distributed as GitHub assets, not through crates.io. All internal
packages must therefore be tested together using local registry patches.
"""

from __future__ import annotations

import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import tarfile

from release_inventory import ReleaseCrate


def extract_crate(archive: Path, destination: Path, crate: ReleaseCrate) -> None:
    """Extract one regular-file Cargo archive; example: ``extract_crate(path, stage, crate)``."""
    prefix = f"{crate.name}-{crate.version}"
    with tarfile.open(archive, "r:gz") as stream:
        for member in stream.getmembers():
            parts = PurePosixPath(member.name).parts
            if (
                not parts
                or parts[0] != prefix
                or ".." in parts
                or not (member.isfile() or member.isdir())
            ):
                raise ValueError(
                    f"archive member {member.name!r}: expected regular files under {prefix}/"
                )
        stream.extractall(destination, filter="data")
    if not (destination / prefix / "Cargo.toml").is_file():
        raise ValueError(f"archive {archive.name!r}: expected {prefix}/Cargo.toml")


def stage_workspace(
    archives: Path, stage: Path, crates: list[ReleaseCrate], lockfile: Path
) -> None:
    """Create a workspace solely from release archives; example: ``stage_workspace(packages, stage, crates, lock)``."""
    stage.mkdir(parents=True, exist_ok=False)
    for crate in crates:
        extract_crate(archives / crate.archive, stage, crate)
        (stage / crate.directory).parent.mkdir(exist_ok=True)
        (stage / f"{crate.name}-{crate.version}").rename(stage / crate.directory)
    write_workspace(stage, crates, lockfile)
    with tarfile.open(archives / "martensite-test-support.tar.gz", "r:gz") as stream:
        stream.extractall(stage, filter="data")


def write_workspace(stage: Path, crates: list[ReleaseCrate], lockfile: Path) -> None:
    """Pin the extracted crate set and toolchain; example: ``write_workspace(stage, crates, lock)``."""
    members = [crate.directory for crate in crates]
    patches = [f'{crate.name} = {{ path = "{crate.directory}" }}' for crate in crates]
    manifest = '[workspace]\nresolver = "3"\nmembers = ' + json.dumps(members)
    manifest += "\n[patch.crates-io]\n" + "\n".join(patches) + "\n"
    (stage / "Cargo.toml").write_text(manifest, encoding="utf-8")
    shutil.copyfile(lockfile, stage / "Cargo.lock")
    shutil.copyfile(
        lockfile.parent / "rust-toolchain.toml", stage / "rust-toolchain.toml"
    )


def file_sha256(path: Path) -> str:
    """Hash bytes without loading the archive into memory; example: ``file_sha256(Path('kernel.crate'))``."""
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def seal_release(
    archives: Path,
    output: Path,
    crates: list[ReleaseCrate],
    revision: str,
    lockfile: Path,
) -> None:
    """Write tested artifacts and exact hashes to a new directory; example: ``seal_release(src, dest, crates, sha, lock)``."""
    output.mkdir(parents=True, exist_ok=False)
    for crate in crates:
        shutil.copyfile(archives / crate.archive, output / crate.archive)
    shutil.copyfile(lockfile, output / "Cargo.lock")
    for name in ("LICENSE", "rust-toolchain.toml"):
        shutil.copyfile(lockfile.parent / name, output / name)
    shutil.copyfile(
        archives / "martensite-test-support.tar.gz",
        output / "martensite-test-support.tar.gz",
    )
    write_receipt(output, crates[0].version, revision)


def write_receipt(output: Path, version: str, revision: str) -> None:
    """Seal the complete artifact inventory; example: ``write_receipt(output, '0.1.0', sha)``."""
    hashes = {path.name: file_sha256(path) for path in sorted(output.iterdir())}
    receipt = {
        "format": 1,
        "repository": "Oblikovati/RustyCADKernel",
        "version": version,
        "revision": revision,
        "artifacts": hashes,
        "validation": "extracted-workspace-tests-and-doctests",
    }
    manifest = output / "martensite-release.json"
    manifest.write_text(
        json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    hashes[manifest.name] = file_sha256(manifest)
    (output / "SHA256SUMS").write_text(
        "".join(f"{digest}  {name}\n" for name, digest in sorted(hashes.items())),
        encoding="utf-8",
    )
