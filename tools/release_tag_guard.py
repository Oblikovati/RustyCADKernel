#!/usr/bin/env python3
"""Reject unverified Martensite tags; example: release_tag_guard.py prepared < updates."""

from __future__ import annotations

import ast
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
from typing import Protocol

from publish_release import artifact_hashes, json_object, validate_assets

PLAN_PATH = "tools/release_verification_plan.py"
REPOSITORY = "Oblikovati/RustyCADKernel"


class GuardGit(Protocol):
    """Inject Git reads; example: verify_tag(tag, oid, receipt, FakeGuardGit())."""

    def read(self, *arguments: str) -> bytes:
        """Return checked Git bytes; example: read('rev-parse', 'HEAD')."""
        ...


class SystemGuardGit:
    """Use the transaction checkout without changing it; example: SystemGuardGit()."""

    def read(self, *arguments: str) -> bytes:
        """Preserve Git failures; example: read('show', 'HEAD:path')."""
        result = subprocess.run(["git", *arguments], capture_output=True, check=True)
        return result.stdout


def require_equal(actual: object, expected: object, label: str) -> None:
    """Reject a mismatched receipt field; example: require_equal(1, 1, 'format')."""
    if type(actual) is not type(expected) or actual != expected:
        raise ValueError(f"{label}={actual!r}: expected {expected!r}")


def object_field(parent: dict[str, object], key: str) -> dict[str, object]:
    """Require structured evidence; example: object_field(receipt, 'packages')."""
    value = parent.get(key)
    if not isinstance(value, dict):
        raise ValueError(f"{key}={value!r}: expected JSON object")
    return value


def evidence_path(base: Path, value: object) -> Path:
    """Resolve receipt-local evidence; example: evidence_path(base, 'tests.log')."""
    if not isinstance(value, str) or not value:
        raise ValueError(f"evidence path={value!r}: expected nonempty path string")
    path = base / value
    if path.is_symlink() or not path.is_file():
        raise ValueError(f"evidence path={str(path)!r}: expected regular file")
    return path


def verify_log(command: object, base: Path) -> None:
    """Bind successful commands to log bytes; example: verify_log(command, base)."""
    if not isinstance(command, dict):
        raise ValueError(f"command={command!r}: expected JSON object")
    require_equal(command.get("exit_code"), 0, "command exit_code")
    argv = command.get("argv")
    if (
        not isinstance(argv, list)
        or not argv
        or any(not isinstance(v, str) or not v for v in argv)
    ):
        raise ValueError(f"command argv={argv!r}: expected nonempty string argv")
    digest = hashlib.sha256(
        evidence_path(base, command.get("log")).read_bytes()
    ).hexdigest()
    require_equal(command.get("sha256"), digest, "command log sha256")


def verify_gate(gate: object, base: Path) -> str:
    """Reject empty or failed gate evidence; example: verify_gate(gate, base)."""
    if not isinstance(gate, dict) or not isinstance(gate.get("name"), str):
        raise ValueError(f"gate={gate!r}: expected named JSON object")
    require_equal(gate.get("exit_code"), 0, "gate exit_code")
    commands = gate.get("commands")
    if not isinstance(commands, list) or not commands:
        raise ValueError(f"gate commands={commands!r}: expected nonempty command list")
    for command in commands:
        verify_log(command, base)
    return str(gate["name"])


def planned_gate_names(plan_bytes: bytes) -> tuple[str, ...]:
    """Read the candidate's literal plan without executing it; example: planned_gate_names(blob)."""
    tree = ast.parse(plan_bytes.decode())
    assignments = [
        node for node in tree.body if isinstance(node, (ast.Assign, ast.AnnAssign))
    ]
    for node in assignments:
        targets = node.targets if isinstance(node, ast.Assign) else [node.target]
        if any(
            isinstance(target, ast.Name) and target.id == "GATE_NAMES"
            for target in targets
        ):
            names = ast.literal_eval(node.value)
            if (
                isinstance(names, tuple)
                and names
                and all(isinstance(name, str) for name in names)
            ):
                return names
    raise ValueError("plan GATE_NAMES: expected nonempty literal string tuple")


def verify_gates(receipt: dict[str, object], plan_bytes: bytes, base: Path) -> None:
    """Require each planned gate exactly once; example: verify_gates(receipt, blob, base)."""
    expected = planned_gate_names(plan_bytes)
    gates = receipt.get("gates")
    if not isinstance(gates, list):
        raise ValueError(f"receipt gates={gates!r}: expected planned gates")
    actual = [verify_gate(gate, base) for gate in gates]
    if len(set(actual)) != len(actual) or sorted(actual) != sorted(expected):
        raise ValueError(f"gate names={actual!r}: expected exactly {expected!r}")


def verify_packages(
    receipt: dict[str, object], tag: str, revision: str, base: Path
) -> None:
    """Independently validate sealed assets; example: verify_packages(r, tag, sha, base)."""
    packages = object_field(receipt, "packages")
    require_equal(packages.get("exit_code"), 0, "packages exit_code")
    manifest = evidence_path(
        base, str(packages.get("artifacts_path", "")) + "/martensite-release.json"
    )
    require_equal(
        packages.get("manifest_sha256"),
        hashlib.sha256(manifest.read_bytes()).hexdigest(),
        "manifest sha256",
    )
    validate_assets(manifest.parent, tag, revision, REPOSITORY)
    require_equal(len(artifact_hashes(manifest.parent)), 37, "sealed asset count")


def verify_annotation(git: GuardGit, oid: str, tag: str) -> None:
    """Reject lightweight and mismatched tag objects; example: verify_annotation(git, oid, tag)."""
    require_equal(
        git.read("cat-file", "-t", oid).decode().strip(), "tag", "release object type"
    )
    header = git.read("cat-file", "-p", oid).decode().split("\n\n", 1)[0].splitlines()
    require_equal(
        [line[4:] for line in header if line.startswith("tag ")],
        [tag],
        "annotated tag name",
    )


def verify_source(git: GuardGit, oid: str, tag: str) -> tuple[str, bytes]:
    """Bind original annotated tag to clean source; example: verify_source(git, oid, tag)."""
    verify_annotation(git, oid, tag)
    revision = git.read("rev-parse", oid + "^{commit}").decode().strip()
    require_equal(
        git.read("rev-parse", "HEAD").decode().strip(), revision, "checkout HEAD"
    )
    require_equal(
        git.read("status", "--porcelain", "--untracked-files=all"),
        b"",
        "checkout status",
    )
    return revision, git.read("show", revision + ":" + PLAN_PATH)


def verify_tag(tag: str, oid: str, receipt_path: Path, git: GuardGit) -> None:
    """Enforce candidate-bound acceptance; example: verify_tag(tag, oid, path, git)."""
    revision, plan_bytes = verify_source(git, oid, tag)
    receipt = json_object(
        evidence_path(receipt_path.parent, receipt_path.name).read_text()
    )
    require_equal(receipt.get("format"), 1, "receipt format")
    require_equal(receipt.get("complete"), True, "receipt complete")
    require_equal(receipt.get("tag"), tag, "receipt tag")
    require_equal(receipt.get("revision"), revision, "receipt revision")
    require_equal(
        receipt.get("verification_plan_sha256"),
        hashlib.sha256(plan_bytes).hexdigest(),
        "plan sha256",
    )
    verify_gates(receipt, plan_bytes, receipt_path.parent)
    verify_packages(receipt, tag, revision, receipt_path.parent)


def guard_transaction(
    state: str, updates: str, receipt: str | None, git: GuardGit
) -> None:
    """Ignore ordinary refs and deletions; example: guard_transaction('prepared', updates, path, git)."""
    if state != "prepared":
        return
    for line in updates.splitlines():
        old, new, reference = line.split()
        match = re.fullmatch(r"refs/tags/(v[0-9]+\.[0-9]+\.[0-9]+)", reference)
        if match is None or set(new) == {"0"}:
            continue
        if not receipt:
            raise ValueError(
                f"release ref={reference!r}: expected MARTENSITE_RELEASE_RECEIPT"
            )
        verify_tag(match[1], new, Path(receipt).absolute(), git)


def main() -> int:
    """Run reference-transaction validation; example: main() from the hook."""
    try:
        guard_transaction(
            sys.argv[1],
            sys.stdin.read(),
            os.environ.get("MARTENSITE_RELEASE_RECEIPT"),
            SystemGuardGit(),
        )
        return 0
    except (
        ValueError,
        OSError,
        SyntaxError,
        TypeError,
        subprocess.CalledProcessError,
    ) as failure:
        print(f"Martensite release tag rejected: {failure}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
