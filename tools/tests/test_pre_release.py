"""The release ref comes after actual verification and package acceptance."""

from pathlib import Path
import subprocess
import unittest

from pre_release import parse_pre_release, verify_then_tag


class FakePreReleaseOperations:
    """Named external boundary records the order and original failures."""

    def __init__(self, failing_phase: str = "") -> None:
        self.failing_phase = failing_phase
        self.events: list[str] = []
        self.checkout = Path("owned-checkout")
        self.assets = Path("sealed-assets")
        self.acceptance = Path("acceptance.json")

    def phase(self, name: str) -> None:
        self.events.append(name)
        if name == self.failing_phase:
            raise subprocess.CalledProcessError(37, ["original", name])

    def prepare(self, tag: str) -> Path:
        self.phase("prepare")
        return self.checkout

    def verify(self, checkout: Path) -> None:
        assert checkout == self.checkout
        self.phase("verify")

    def package(self, checkout: Path, tag: str) -> Path:
        assert checkout == self.checkout
        self.phase("package")
        return self.assets

    def receipt(self, checkout: Path, tag: str, assets: Path) -> Path:
        assert checkout == self.checkout and assets == self.assets
        self.phase("receipt")
        return self.acceptance

    def tag(self, checkout: Path, tag: str, receipt: Path) -> None:
        assert checkout == self.checkout and receipt == self.acceptance
        self.phase("tag")

    def dispose(self, checkout: Path) -> None:
        assert checkout == self.checkout
        self.phase("dispose")


class PreReleaseOrderTests(unittest.TestCase):
    def test_complete_verification_precedes_tag_creation(self) -> None:
        boundary = FakePreReleaseOperations()
        result = verify_then_tag("v0.2.7", boundary)
        self.assertEqual(result, boundary.acceptance)
        self.assertEqual(
            boundary.events,
            ["prepare", "verify", "package", "receipt", "tag", "dispose"],
        )

    def test_verification_package_or_receipt_error_creates_no_tag(self) -> None:
        for failed in ("verify", "package", "receipt"):
            with self.subTest(failed=failed):
                boundary = FakePreReleaseOperations(failed)
                with self.assertRaises(subprocess.CalledProcessError) as error:
                    verify_then_tag("v0.2.7", boundary)
                self.assertEqual(error.exception.returncode, 37)
                self.assertNotIn("tag", boundary.events)
                self.assertEqual(boundary.events[-1], "dispose")

    def test_guard_rejection_keeps_original_failure_and_disposes_checkout(self) -> None:
        boundary = FakePreReleaseOperations("tag")
        with self.assertRaises(subprocess.CalledProcessError) as error:
            verify_then_tag("v0.2.7", boundary)
        self.assertEqual(error.exception.cmd, ["original", "tag"])
        self.assertEqual(boundary.events[-1], "dispose")

    def test_prepare_failure_cannot_tag_or_dispose_uncreated_checkout(self) -> None:
        boundary = FakePreReleaseOperations("prepare")
        with self.assertRaises(subprocess.CalledProcessError):
            verify_then_tag("v0.2.7", boundary)
        self.assertEqual(boundary.events, ["prepare"])

    def test_failure_does_not_prevent_fresh_success(self) -> None:
        boundary = FakePreReleaseOperations("package")
        with self.assertRaises(subprocess.CalledProcessError):
            verify_then_tag("v0.2.7", boundary)
        boundary.failing_phase = ""
        boundary.events.clear()
        self.assertEqual(verify_then_tag("v0.2.7", boundary), boundary.acceptance)
        self.assertIn("tag", boundary.events)

    def test_cli_requires_explicit_tag_and_oracle_source(self) -> None:
        parsed = parse_pre_release(
            ["--tag", "v0.2.7", "--source", "../OCCT", "--root", "candidate"]
        )
        self.assertEqual(
            (parsed.tag, parsed.source, parsed.root),
            ("v0.2.7", Path("../OCCT"), Path("candidate")),
        )


if __name__ == "__main__":
    unittest.main()
