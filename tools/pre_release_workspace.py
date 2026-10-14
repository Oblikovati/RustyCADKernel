"""A disposable committed checkout keeps user edits outside release artifacts."""

from __future__ import annotations

import os
from pathlib import Path
import re
import subprocess
import tempfile
import tomllib


def checked_git(root: Path, *arguments: str) -> str:
    """Run one owned Git operation; example: checked_git(root, 'rev-parse', 'HEAD')."""
    argv = ["git", *arguments]
    return subprocess.check_output(argv, cwd=root, text=True).strip()


def require_new_release_tag(root: Path, tag: str) -> str:
    """Reject overwritten or mismatched identities; example: require_new_release_tag(root, 'v0.2.7')."""
    if not re.fullmatch(
        r"v(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)", tag
    ):
        raise ValueError(f"release tag={tag!r}: expected vMAJOR.MINOR.PATCH")
    revision = checked_git(root, "rev-parse", "HEAD")
    require_committed_version(root, revision, tag)
    require_absent_tag(root, tag)
    return revision


def require_committed_version(root: Path, revision: str, tag: str) -> None:
    """Match the committed package identity; example: require_committed_version(root, sha, tag)."""
    manifest = tomllib.loads(checked_git(root, "show", revision + ":Cargo.toml"))
    expected = "v" + manifest["workspace"]["package"]["version"]
    if tag != expected:
        raise ValueError(
            f"release tag={tag!r}: expected committed version {expected!r}"
        )


def require_absent_tag(root: Path, tag: str) -> None:
    """Never overwrite a release ref; example: require_absent_tag(root, tag)."""
    exists = subprocess.run(
        ["git", "show-ref", "--verify", "--quiet", "refs/tags/" + tag], cwd=root
    )
    if exists.returncode != 1:
        raise ValueError(
            f"release tag={tag!r}, lookup exit={exists.returncode}: expected absent local tag"
        )


def install_release_hook(root: Path) -> None:
    """Preserve unrelated hooks while enforcing local tags; example: install_release_hook(root)."""
    hook_directory = (root / ".githooks").resolve()
    require_owned_hook_configuration(root, hook_directory)
    require_executable_release_hook(hook_directory)
    subprocess.run(
        ["git", "config", "--local", "core.hooksPath", str(hook_directory)],
        cwd=root,
        check=True,
    )


def require_owned_hook_configuration(root: Path, hook_directory: Path) -> None:
    """Leave pre-existing custom hooks intact; example: require_owned_hook_configuration(root, directory)."""
    result = subprocess.run(
        ["git", "config", "--get", "core.hooksPath"],
        cwd=root,
        capture_output=True,
        text=True,
    )
    configured = result.stdout.strip()
    if configured and (root / configured).resolve() != hook_directory:
        raise ValueError(
            f"core.hooksPath={configured!r}: expected unset or owned hook directory {str(hook_directory)!r}"
        )
    if not configured:
        require_no_custom_default_hooks(root)


def require_no_custom_default_hooks(root: Path) -> None:
    """Refuse to replace another local hook policy; example: require_no_custom_default_hooks(root)."""
    common = (root / checked_git(root, "rev-parse", "--git-common-dir")).resolve()
    active = [
        path.name
        for path in (common / "hooks").iterdir()
        if path.is_file()
        and not path.name.endswith(".sample")
        and os.access(path, os.X_OK)
    ]
    if active:
        raise ValueError(
            f"default Git hooks={active!r}: expected no custom executable hooks to replace"
        )


def require_executable_release_hook(hook_directory: Path) -> None:
    """Reject incomplete policy installation; example: require_executable_release_hook(directory)."""
    hook = hook_directory / "reference-transaction"
    if not hook.is_file() or not os.access(hook, os.X_OK):
        raise ValueError(
            f"release hook={str(hook)!r}: expected committed executable hook"
        )


def create_release_checkout(root: Path, tag: str) -> tuple[Path, Path, str]:
    """Freeze committed HEAD in a clean checkout; example: create_release_checkout(root, tag)."""
    revision = require_new_release_tag(root, tag)
    install_release_hook(root)
    evidence_root = root / "target/pre-release" / tag
    evidence_root.mkdir(parents=True, exist_ok=True)
    evidence = Path(tempfile.mkdtemp(prefix=revision[:12] + "-", dir=evidence_root))
    checkout = evidence / "checkout"
    subprocess.run(
        ["git", "worktree", "add", "--quiet", "--detach", str(checkout), revision],
        cwd=root,
        check=True,
    )
    return checkout, evidence, revision


def delete_release_checkout(root: Path, checkout: Path) -> None:
    """Physically delete only the owned temporary tree; example: delete_release_checkout(root, checkout)."""
    if not checkout.is_dir():
        return
    # The runner awaits each command, so no child uses this checkout at disposal.
    subprocess.run(
        ["git", "worktree", "remove", "--force", str(checkout)], cwd=root, check=True
    )
