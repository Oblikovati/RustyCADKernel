"""Release tag acceptance and ordinary Git transaction regressions."""

from __future__ import annotations

import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from release_archives import write_receipt
from release_tag_guard import (
    guard_transaction,
    planned_gate_names,
    plan_command_roster,
    verify_tag,
)

REVISION = "a" * 40
OID = "b" * 40
PLAN = b"""from types import SimpleNamespace as Record
GATE_NAMES = ('tests', 'docs', 'coverage')
def verification_gates(root, source):
    def command(argv, env=(), stdout_path=None):
        return Record(argv=argv, env=env, stdout_path=stdout_path)
    checks = (command(('python3', 'discovery')), command(('python3', 'format')), command(('cargo', 'lint', '-Dwarnings')))
    docs = (command(('cargo', 'doc'), (('RUSTDOCFLAGS', '-D warnings'),)),)
    coverage = (command(('cargo', 'llvm-cov', '-E', 'not test(slow)'), stdout_path=root / 'coverage.log'),)
    return (Record(name='tests', commands=checks), Record(name='docs', commands=docs), Record(name='coverage', commands=coverage))
"""


class FakeGuardGit:
    def __init__(self) -> None:
        self.status = b""
        self.head = REVISION
        self.kind = "tag"
        self.root = Path.cwd()

    def read(self, *arguments: str) -> bytes:
        responses = {
            ("cat-file", "-t", OID): self.kind.encode(),
            ("cat-file", "-p", OID): b"object "
            + REVISION.encode()
            + b"\ntype commit\ntag v0.2.8\n\nRelease\n",
            ("rev-parse", OID + "^{commit}"): REVISION.encode(),
            ("rev-parse", "HEAD"): self.head.encode(),
            ("rev-parse", "--show-toplevel"): str(self.root).encode(),
            ("status", "--porcelain", "--untracked-files=all"): self.status,
            ("show", REVISION + ":tools/release_verification_plan.py"): PLAN,
        }
        return responses[arguments]


class ReleaseEvidenceFixture:
    def __init__(self, root: Path) -> None:
        self.root = root
        self.assets = root / "assets"
        self.assets.mkdir()
        for index in range(35):
            (self.assets / f"archive-{index}.crate").write_bytes(b"accepted package")
        write_receipt(self.assets, "0.2.8", REVISION)
        (root / "tests.log").write_text("passed\n")
        self.receipt_path = root / "acceptance.json"
        self.receipt = self.acceptance()
        self.write()

    def acceptance(self) -> dict[str, object]:
        gates, manifest_hash = self.gate_evidence()
        return {
            "format": 1,
            "tag": "v0.2.8",
            "revision": REVISION,
            "complete": True,
            "occt_source": "/fixture/OCCT",
            "verification_plan_sha256": hashlib.sha256(PLAN).hexdigest(),
            "gates": gates,
            "packages": {
                "exit_code": 0,
                "artifacts_path": "assets",
                "manifest_sha256": manifest_hash,
            },
        }

    def gate_evidence(self) -> tuple[list[dict[str, object]], str]:
        roster = plan_command_roster(PLAN, Path.cwd(), Path("/fixture/OCCT"))
        gates = []
        for name, commands in roster.items():
            evidence = [
                dict(
                    command,
                    exit_code=0,
                    log="tests.log",
                    sha256=hashlib.sha256(b"passed\n").hexdigest(),
                )
                for command in commands
            ]
            gates.append({"name": name, "exit_code": 0, "commands": evidence})
        manifest_hash = hashlib.sha256(
            (self.assets / "martensite-release.json").read_bytes()
        ).hexdigest()
        return gates, manifest_hash

    def write(self) -> None:
        self.receipt_path.write_text(json.dumps(self.receipt))


class ReleaseGuardTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.fixture = ReleaseEvidenceFixture(Path(temporary.name))
        self.git = FakeGuardGit()

    def verify(self) -> None:
        verify_tag("v0.2.8", OID, self.fixture.receipt_path, self.git)

    def test_success_and_nonrelease_transactions(self) -> None:
        self.verify()
        for state, ref, new in [
            ("committed", "refs/tags/v0.2.8", OID),
            ("prepared", "refs/heads/main", OID),
            ("prepared", "refs/tags/v0.2.8", "0" * 40),
        ]:
            guard_transaction(state, f"{REVISION} {new} {ref}", None, self.git)

    def test_missing_receipt_and_malformed_plan(self) -> None:
        with self.assertRaisesRegex(ValueError, "MARTENSITE_RELEASE_RECEIPT"):
            guard_transaction(
                "prepared", f"{'0' * 40} {OID} refs/tags/v0.2.8", None, self.git
            )
        for plan in [b"GATE_NAMES = ()", b"GATE_NAMES = function()", b"OTHER = 1"]:
            with self.assertRaises((ValueError, TypeError)):
                planned_gate_names(plan)

    def test_failed_stale_and_missing_receipt_fields(self) -> None:
        original = copy.deepcopy(self.fixture.receipt)
        for field, value in [
            ("complete", False),
            ("format", True),
            ("tag", "v0.2.9"),
            ("revision", OID),
            ("verification_plan_sha256", "0" * 64),
            ("gates", []),
        ]:
            self.fixture.receipt = copy.deepcopy(original)
            self.fixture.receipt[field] = value
            self.fixture.write()
            with self.assertRaises(ValueError):
                self.verify()

    def test_failed_empty_duplicate_and_tampered_commands(self) -> None:
        original = copy.deepcopy(self.fixture.receipt)
        for mutation in ("failed", "empty", "duplicate", "argv", "log"):
            self.fixture.receipt = copy.deepcopy(original)
            gates = self.fixture.receipt["gates"]
            if mutation == "failed":
                gates[0]["exit_code"] = 1
            if mutation == "empty":
                gates[0]["commands"] = []
            if mutation == "duplicate":
                gates[1]["name"] = "tests"
            if mutation == "argv":
                gates[0]["commands"][0]["argv"] = []
            if mutation == "log":
                gates[0]["commands"][0]["sha256"] = "0" * 64
            self.fixture.write()
            with self.assertRaises(ValueError):
                self.verify()

    def test_wrong_source_and_lightweight_tags(self) -> None:
        for attribute, value in [
            ("status", b" M source.rs"),
            ("head", OID),
            ("kind", "commit"),
        ]:
            self.git = FakeGuardGit()
            setattr(self.git, attribute, value)
            with self.assertRaises(ValueError):
                self.verify()

    def test_tampered_assets_and_missing_log(self) -> None:
        (self.fixture.assets / "archive-0.crate").write_bytes(b"tampered")
        with self.assertRaisesRegex(ValueError, "checksums"):
            self.verify()
        (self.fixture.root / "tests.log").unlink()
        with self.assertRaisesRegex(ValueError, "regular file"):
            self.verify()

    def test_exact_command_rosters(self) -> None:
        original = copy.deepcopy(self.fixture.receipt)
        for alteration in (
            "omitted",
            "formatter",
            "echo",
            "lint",
            "coverage",
            "env",
            "stdout",
        ):
            self.fixture.receipt = copy.deepcopy(original)
            self.alter_roster(alteration)
            self.fixture.write()
            with self.assertRaisesRegex(ValueError, "command roster"):
                self.verify()

    def test_release_updates_are_immutable(self) -> None:
        with self.assertRaisesRegex(ValueError, "immutable"):
            guard_transaction(
                "prepared",
                f"{REVISION} {OID} refs/tags/v0.2.8",
                str(self.fixture.receipt_path),
                self.git,
            )

    def alter_roster(self, alteration: str) -> None:
        gates = self.fixture.receipt["gates"]
        if alteration == "omitted":
            gates[0]["commands"].pop()
        if alteration == "formatter":
            gates[0]["commands"].pop(1)
        if alteration == "echo":
            gates[0]["commands"][0]["argv"] = ["echo", "passed"]
        if alteration == "lint":
            gates[0]["commands"][2]["argv"].pop()
        if alteration == "coverage":
            gates[2]["commands"][0]["argv"][-1] = "all()"
        if alteration == "env":
            gates[1]["commands"][0]["env"] = {}
        if alteration == "stdout":
            gates[2]["commands"][0]["stdout_path"] = None


class RealHookIntegration(unittest.TestCase):
    def test_hook_rejects_release_allows_branch_and_delete(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.git(root, "init", "-q")
            self.git(root, "config", "user.email", "fixture@example.invalid")
            self.git(root, "config", "user.name", "Named release fixture")
            self.git(root, "commit", "--allow-empty", "-qm", "fixture")
            hook = Path(__file__).resolve().parents[2] / ".githooks"
            self.git(root, "config", "core.hooksPath", str(hook))
            self.git(root, "branch", "ordinary")
            result = self.git(root, "tag", "-a", "v0.2.8", "-m", "release", check=False)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("MARTENSITE_RELEASE_RECEIPT", result.stderr)
            self.git(root, "tag", "ordinary-tag")
            self.git(root, "tag", "-d", "ordinary-tag")

    def test_real_annotated_tag_acceptance_and_asset_rejection(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root = base / "repository"
            root.mkdir()
            (root / "tools").mkdir()
            (root / "tools/release_verification_plan.py").write_bytes(PLAN)
            self.git(root, "init", "-q")
            self.git(root, "config", "user.email", "fixture@example.invalid")
            self.git(root, "config", "user.name", "Named release fixture")
            self.git(root, "add", ".")
            self.git(root, "commit", "-qm", "source")
            self.accept_release(root, base)

    def accept_release(self, root: Path, base: Path) -> None:
        fixture = ReleaseEvidenceFixture(base)
        revision = self.git(root, "rev-parse", "HEAD").stdout.strip()
        (fixture.assets / "SHA256SUMS").unlink()
        (fixture.assets / "martensite-release.json").unlink()
        write_receipt(fixture.assets, "0.2.8", revision)
        fixture.receipt["revision"] = revision
        fixture.receipt["gates"][2]["commands"][0]["stdout_path"] = str(
            root / "coverage.log"
        )
        fixture.receipt["packages"]["manifest_sha256"] = hashlib.sha256(
            (fixture.assets / "martensite-release.json").read_bytes()
        ).hexdigest()
        fixture.write()
        hook = Path(__file__).resolve().parents[2] / ".githooks"
        self.git(root, "config", "core.hooksPath", str(hook))
        self.check_release_attempts(root, fixture)

    def check_release_attempts(
        self, root: Path, fixture: ReleaseEvidenceFixture
    ) -> None:
        result = self.tag_with_receipt(root, fixture.receipt_path)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.git(root, "tag", "-d", "v0.2.8")
        (fixture.assets / "archive-0.crate").write_bytes(b"tampered")
        self.assertNotEqual(
            self.tag_with_receipt(root, fixture.receipt_path).returncode, 0
        )

    def tag_with_receipt(
        self, root: Path, receipt: Path
    ) -> subprocess.CompletedProcess[str]:
        environment = dict(os.environ, MARTENSITE_RELEASE_RECEIPT=str(receipt))
        return subprocess.run(
            ["git", "tag", "-a", "v0.2.8", "-m", "release"],
            cwd=root,
            env=environment,
            capture_output=True,
            text=True,
        )

    def git(
        self, root: Path, *arguments: str, check: bool = True
    ) -> subprocess.CompletedProcess[str]:
        environment = {
            key: value
            for key, value in os.environ.items()
            if key != "MARTENSITE_RELEASE_RECEIPT"
        }
        return subprocess.run(
            ["git", *arguments],
            cwd=root,
            env=environment,
            text=True,
            capture_output=True,
            check=check,
        )


if __name__ == "__main__":
    unittest.main()
