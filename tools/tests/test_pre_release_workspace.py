"""Disposable real repositories verify checkout and hook ownership boundaries."""

from pathlib import Path
import subprocess
import tempfile
import unittest

from pre_release_workspace import (
    checked_git,
    create_release_checkout,
    delete_release_checkout,
    install_release_hook,
    require_absent_tag,
    require_committed_version,
    require_new_release_tag,
)


class PreReleaseRepositoryFixture:
    """Own a tiny Git repository; no host Git settings or user files change."""

    def __init__(self, root: Path) -> None:
        self.root = root
        self.run("init", "--quiet")
        self.run("config", "user.name", "Release fixture")
        self.run("config", "user.email", "fixture@example.invalid")
        (root / "Cargo.toml").write_text('[workspace.package]\nversion="0.2.7"\n')
        (root / ".gitignore").write_text("target/\n")
        hook = root / ".githooks/reference-transaction"
        hook.parent.mkdir()
        hook.write_text("#!/bin/sh\nexit 0\n")
        hook.chmod(0o755)
        self.run("add", ".")
        self.run("commit", "--quiet", "-m", "fixture")

    def run(self, *arguments: str) -> str:
        return subprocess.check_output(
            ["git", *arguments], cwd=self.root, text=True
        ).strip()


class PreReleaseWorkspaceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.fixture = PreReleaseRepositoryFixture(Path(self.temporary.name))
        self.root = self.fixture.root

    def test_new_tag_requires_committed_version_and_rejects_existing_or_invalid_tag(
        self,
    ) -> None:
        revision = require_new_release_tag(self.root, "v0.2.7")
        require_committed_version(self.root, revision, "v0.2.7")
        require_absent_tag(self.root, "v0.2.7")
        for tag in ("v0.2.6", "v00.2.7", "main", "v0.2.7;false"):
            with self.assertRaises(ValueError):
                require_new_release_tag(self.root, tag)
        self.fixture.run("tag", "v0.2.7")
        with self.assertRaisesRegex(ValueError, "absent"):
            require_new_release_tag(self.root, "v0.2.7")

    def test_owned_hook_installation_is_idempotent(self) -> None:
        install_release_hook(self.root)
        install_release_hook(self.root)
        self.assertEqual(
            checked_git(self.root, "config", "--get", "core.hooksPath"),
            str(self.root / ".githooks"),
        )

    def test_custom_configured_hooks_are_preserved(self) -> None:
        self.fixture.run("config", "core.hooksPath", "other-hooks")
        with self.assertRaisesRegex(ValueError, "expected unset"):
            install_release_hook(self.root)
        self.assertEqual(
            self.fixture.run("config", "--get", "core.hooksPath"), "other-hooks"
        )

    def test_custom_default_hook_is_not_discarded(self) -> None:
        custom = self.root / ".git/hooks/pre-commit"
        custom.write_text("#!/bin/sh\nexit 0\n")
        custom.chmod(0o755)
        with self.assertRaisesRegex(ValueError, "custom executable"):
            install_release_hook(self.root)
        self.assertTrue(custom.is_file())

    def test_missing_or_nonexecutable_guard_cannot_install(self) -> None:
        hook = self.root / ".githooks/reference-transaction"
        hook.chmod(0o644)
        with self.assertRaisesRegex(ValueError, "executable"):
            install_release_hook(self.root)
        hook.unlink()
        with self.assertRaisesRegex(ValueError, "executable"):
            install_release_hook(self.root)

    def test_checkout_uses_committed_head_and_leaves_user_edits_untouched(self) -> None:
        (self.root / "Cargo.toml").write_text("user edit\n")
        checkout, evidence, revision = create_release_checkout(self.root, "v0.2.7")
        self.assertEqual(checked_git(checkout, "rev-parse", "HEAD"), revision)
        self.assertEqual(checked_git(checkout, "status", "--porcelain"), "")
        (evidence / "failed-gate.log").write_text("durable diagnostic\n")
        delete_release_checkout(self.root, checkout)
        delete_release_checkout(self.root, checkout)
        self.assertFalse(checkout.exists())
        self.assertTrue((evidence / "failed-gate.log").is_file())
        self.assertEqual((self.root / "Cargo.toml").read_text(), "user edit\n")


if __name__ == "__main__":
    unittest.main()
