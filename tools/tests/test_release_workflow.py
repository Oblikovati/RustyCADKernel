"""Release orchestration binds immutable inputs and retains red gates and evidence."""

from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[2]


class ReleaseWorkflowTests(unittest.TestCase):
    def setUp(self) -> None:
        self.release = (ROOT / ".github/workflows/release.yml").read_text()
        self.owner = (ROOT / ".github/workflows/ci.yml").read_text()

    def test_manual_dispatch_resolves_a_tag_before_owner_checks(self) -> None:
        self.assertIn("release_tag:", self.release)
        self.assertIn(
            'revision=$(git rev-parse "refs/tags/$RELEASE_TAG^{commit}")', self.release
        )
        self.assertIn("source_ref: ${{ needs.resolve.outputs.revision }}", self.release)
        self.assertIn("needs: [resolve, owner-checks]", self.release)
        self.assertNotIn("RELEASE_REF:", self.release)

    def test_dispatch_tag_guard_rejects_branches_and_malformed_versions(self) -> None:
        guard = next(
            line.strip()
            for line in self.release.splitlines()
            if '[[ "$RELEASE_TAG" =~' in line
        )
        for tag in ["v0.2.5", "main", "refs/tags/v0.2.5", "v01.2.5", "v0.2.5; false"]:
            with self.subTest(tag=tag):
                result = subprocess.run(
                    ["bash", "-c", guard], env={"RELEASE_TAG": tag}, capture_output=True
                )
                self.assertEqual(result.returncode == 0, tag == "v0.2.5")

    def test_all_owner_checkouts_use_the_resolved_revision(self) -> None:
        self.assertEqual(
            self.owner.count("ref: ${{ inputs.source_ref || github.sha }}"), 6
        )
        self.assertIn("ref: ${{ needs.resolve.outputs.revision }}", self.release)
        self.assertIn("RELEASE_TAG: ${{ needs.resolve.outputs.tag }}", self.release)
        self.assertIn("needs: [resolve, package]", self.release)

    def test_diagnostic_upload_runs_after_a_failed_package_gate(self) -> None:
        self.assertIn(
            "name: Retain release runner context after package tests\n        if: always()",
            self.release,
        )
        self.assertIn(
            "if: always()\n        with:\n          name: martensite-release-diagnostics",
            self.release,
        )
        self.assertIn("name: martensite-release\n", self.release)
        self.assertIn(
            "target/releases/${{ needs.resolve.outputs.tag }}/*", self.release
        )

    def test_logged_test_failure_cannot_turn_into_success(self) -> None:
        self.assertIn("set -euo pipefail", self.release)
        result = subprocess.run(
            ["bash", "-c", "set -euo pipefail; false | cat"], capture_output=True
        )
        self.assertNotEqual(result.returncode, 0)

    def test_concurrency_follows_target_tag_for_dispatch_and_tag_push(self) -> None:
        self.assertIn(
            "group: martensite-release-${{ inputs.release_tag || github.ref_name }}",
            self.release,
        )
        self.assertIn("cancel-in-progress: false", self.release)


if __name__ == "__main__":
    unittest.main()
