#!/usr/bin/env python3
"""Publish verified Martensite assets; example: publish_release.py --help."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile
from typing import Protocol

from release_archives import file_sha256


class GitHubCommands(Protocol):
    """Owned checked GitHub boundary; example: publish(..., FakeGitHub())."""

    def run(self, arguments: list[str]) -> str:
        """Execute checked arguments; example: run(['gh', 'release', 'view', tag])."""
        ...

    def release(self, repository: str, tag: str) -> dict[str, object] | None:
        """Read a release or confirmed absence; example: release(repo, 'v0.2.4')."""
        ...


class SystemGitHub:
    """Real CLI transport, injected only by the entry point; example: SystemGitHub()."""

    def run(self, arguments: list[str]) -> str:
        """Propagate CLI failures unchanged; example: run(['gh', '--version'])."""
        result = subprocess.run(arguments, text=True, capture_output=True)
        return checked_output(result)

    def release(self, repository: str, tag: str) -> dict[str, object] | None:
        """Distinguish absent releases from auth/network failure; example: release(repo, tag)."""
        fields = "isDraft,tagName,assets"
        arguments = ["gh", "release", "view", tag]
        arguments += ["--repo", repository, "--json", fields]
        result = subprocess.run(arguments, text=True, capture_output=True)
        if result.returncode and result.stderr.strip() == "release not found":
            return None
        receipt = json_object(checked_output(result))
        return {
            "draft": receipt.get("isDraft"),
            "tag_name": receipt.get("tagName"),
            "assets": receipt.get("assets"),
        }


def checked_output(result: subprocess.CompletedProcess[str]) -> str:
    """Retain actual CLI error diagnostics and status; example: checked_output(completed)."""
    if result.returncode:
        print(result.stdout, end="", file=sys.stderr)
        print(result.stderr, end="", file=sys.stderr)
    result.check_returncode()
    return result.stdout


def json_object(serialized: str) -> dict[str, object]:
    """Require a JSON object; example: json_object('{"format": 1}')."""
    decoded: object = json.loads(serialized)
    if not isinstance(decoded, dict):
        raise ValueError(f"JSON {decoded!r}: expected an object")
    return decoded


def artifact_hashes(artifacts: Path) -> dict[str, str]:
    """Hash only regular flat assets; example: artifact_hashes(Path('release'))."""
    entries = sorted(artifacts.iterdir())
    if not entries or any(
        entry.is_symlink() or not entry.is_file() for entry in entries
    ):
        raise ValueError(
            f"artifacts {str(artifacts)!r}: expected nonempty regular flat files"
        )
    return {entry.name: file_sha256(entry) for entry in entries}


def checksum_inventory(artifacts: Path) -> dict[str, str]:
    """Reject malformed or duplicate sealed entries; example: checksum_inventory(path)."""
    inventory: dict[str, str] = {}
    for line in (artifacts / "SHA256SUMS").read_text().splitlines():
        match = re.fullmatch(r"([0-9a-f]{64})  ([A-Za-z0-9_.-]+)", line)
        if match is None or match[2] in inventory:
            raise ValueError(
                f"checksum {line!r}: expected a unique SHA256 and flat filename"
            )
        inventory[match[2]] = match[1]
    return inventory


def validate_assets(artifacts: Path, tag: str, revision: str, repository: str) -> None:
    """Bind sealed bytes to their owner; example: validate_assets(path, tag, sha, repo)."""
    hashes = artifact_hashes(artifacts)
    checksums = checksum_inventory(artifacts)
    expected = {name: digest for name, digest in hashes.items() if name != "SHA256SUMS"}
    if checksums != expected:
        raise ValueError(
            f"checksums {checksums!r}: expected exact sealed hashes {expected!r}"
        )
    receipt = json_object((artifacts / "martensite-release.json").read_text())
    validate_receipt_owner(receipt, tag, revision, repository)
    validate_receipt_assets(receipt, hashes)


def validate_receipt_owner(
    receipt: dict[str, object], tag: str, revision: str, repository: str
) -> None:
    """Check owner identity independently of byte hashes; example: validate_receipt_owner(r, tag, sha, repo)."""
    expected = {
        "format": 1,
        "repository": repository,
        "version": tag[1:],
        "revision": revision,
    }
    for key, value in expected.items():
        if receipt.get(key) != value or (
            key == "format" and type(receipt.get(key)) is not int
        ):
            raise ValueError(f"receipt {key}={receipt.get(key)!r}: expected {value!r}")


def validate_receipt_assets(receipt: dict[str, object], hashes: dict[str, str]) -> None:
    """Require the tested sealed inventory; example: validate_receipt_assets(receipt, hashes)."""
    excluded = {"SHA256SUMS", "martensite-release.json"}
    expected = {name: digest for name, digest in hashes.items() if name not in excluded}
    if receipt.get("artifacts") != expected:
        raise ValueError(
            f"receipt artifacts={receipt.get('artifacts')!r}: expected {expected!r}"
        )
    validation = "extracted-workspace-tests-and-doctests"
    if receipt.get("validation") != validation:
        raise ValueError(
            f"receipt validation={receipt.get('validation')!r}: expected {validation}"
        )


def validate_identity(tag: str, revision: str, repository: str) -> None:
    """Reject unsafe CLI/API identities; example: validate_identity(tag, sha, repo)."""
    patterns = (
        (tag, r"v[0-9]+\.[0-9]+\.[0-9]+"),
        (revision, r"[0-9a-f]{40}"),
        (repository, r"[A-Za-z0-9][A-Za-z0-9_.-]*/[A-Za-z0-9][A-Za-z0-9_.-]*"),
    )
    for value, pattern in patterns:
        if re.fullmatch(pattern, value) is None:
            raise ValueError(f"identity {value!r}: expected {pattern!r}")


def verify_remote_tag(
    commands: GitHubCommands, tag: str, revision: str, repository: str
) -> None:
    """Peel annotated tags before writes; example: verify_remote_tag(fake, tag, sha, repo)."""
    endpoint = f"repos/{repository}/git/ref/tags/{tag}"
    current = json_object(commands.run(["gh", "api", endpoint])).get("object")
    for _ in range(16):
        if not isinstance(current, dict):
            break
        if current.get("type") == "commit" and current.get("sha") == revision:
            return
        if current.get("type") != "tag" or not re.fullmatch(
            r"[0-9a-f]{40}", str(current.get("sha"))
        ):
            break
        endpoint = f"repos/{repository}/git/tags/{current['sha']}"
        current = json_object(commands.run(["gh", "api", endpoint])).get("object")
    raise ValueError(
        f"remote tag {tag!r} object={current!r}: expected commit {revision}"
    )


def verify_draft(
    commands: GitHubCommands,
    release: dict[str, object],
    artifacts: Path,
    tag: str,
    repository: str,
) -> None:
    """Resume only identical unpublished assets; example: verify_draft(fake, draft, path, tag, repo)."""
    if release.get("draft") is not True or release.get("tag_name") != tag:
        raise ValueError(
            f"existing release {release!r}: expected an unpublished draft for {tag}"
        )
    validate_remote_names(release.get("assets"), artifacts)
    verify_remote_bytes(commands, artifacts, tag, repository)


def verify_remote_bytes(
    commands: GitHubCommands, artifacts: Path, tag: str, repository: str
) -> None:
    """Hash downloaded draft bytes before publication; example: verify_remote_bytes(fake, path, tag, repo)."""
    with tempfile.TemporaryDirectory(prefix="martensite-draft-check-") as temporary:
        commands.run(
            ["gh", "release", "download", tag, "--repo", repository, "--dir", temporary]
        )
        if artifact_hashes(Path(temporary)) != artifact_hashes(artifacts):
            raise ValueError(
                f"draft {tag!r}: expected byte-identical sealed remote assets"
            )


def validate_remote_names(assets: object, artifacts: Path) -> None:
    """Reject missing/duplicate remote uploads; example: validate_remote_names(draft['assets'], path)."""
    if not isinstance(assets, list) or any(
        not isinstance(asset, dict) for asset in assets
    ):
        raise ValueError(f"remote assets {assets!r}: expected asset objects")
    names = [asset.get("name") for asset in assets]
    if any(not isinstance(name, str) for name in names):
        raise ValueError(f"remote assets {names!r}: expected string asset names")
    if len(names) != len(set(names)) or set(names) != set(artifact_hashes(artifacts)):
        raise ValueError(f"remote assets {names!r}: expected exact sealed asset names")


def publish(
    artifacts: Path, tag: str, revision: str, repository: str, commands: GitHubCommands
) -> None:
    """Publish or recover one verified draft; example: publish(path, tag, sha, repo, fake)."""
    validate_identity(tag, revision, repository)
    validate_assets(artifacts, tag, revision, repository)
    verify_remote_tag(commands, tag, revision, repository)
    release = commands.release(repository, tag)
    if release is not None:
        verify_draft(commands, release, artifacts, tag, repository)
    else:
        create_draft(commands, artifacts, tag, repository)
    # Retrying after a failed edit never uploads or overwrites an existing asset.
    verify_remote_tag(commands, tag, revision, repository)
    publish_draft(commands, tag, repository)


def create_draft(
    commands: GitHubCommands, artifacts: Path, tag: str, repository: str
) -> None:
    """Upload exactly verified assets into a new draft; example: create_draft(fake, path, tag, repo)."""
    assets = [str(path.resolve()) for path in sorted(artifacts.iterdir())]
    arguments = ["gh", "release", "create", tag, "--repo", repository, "--verify-tag"]
    arguments += [
        "--draft",
        "--title",
        f"Martensite {tag}",
        "--generate-notes",
        *assets,
    ]
    commands.run(arguments)


def publish_draft(commands: GitHubCommands, tag: str, repository: str) -> None:
    """Make the verified draft visible; example: publish_draft(fake, tag, repo)."""
    arguments = ["gh", "release", "edit", tag, "--repo", repository]
    arguments += ["--draft=false", "--prerelease=false", "--latest=false"]
    commands.run(arguments)


def main() -> None:
    """Parse explicit workflow ownership; example: publish_release.py --help."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--artifacts", required=True, type=Path)
    for name in ("tag", "revision", "repository"):
        parser.add_argument(f"--{name}", required=True)
    arguments = parser.parse_args()
    publish(
        arguments.artifacts,
        arguments.tag,
        arguments.revision,
        arguments.repository,
        SystemGitHub(),
    )


if __name__ == "__main__":
    main()
