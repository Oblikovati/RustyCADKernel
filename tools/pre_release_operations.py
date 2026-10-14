"""Execute workflow parity and package gates in a disposable release checkout."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
from collections.abc import Mapping

from pre_release_execution import (
    CandidateEnvironmentCommands,
    local_gate_environment,
    preserve_failure_diagnostics,
    require_cargo_tool_versions,
    verify_workflow_gates,
)
from pre_release_workspace import (
    checked_git,
    create_release_checkout,
    delete_release_checkout,
)
from publish_release import validate_assets
from release_inventory import clean_release_revision
from release_packages import build_release_candidate


class NativePreReleaseOperations:
    """Keep the tag after all successful evidence; example: NativePreReleaseOperations(root, source)."""

    def __init__(self, root: Path, source: Path, inherited: Mapping[str, str]) -> None:
        self.root = root
        self.source = source
        self.inherited = dict(inherited)
        self.evidence = root
        self.revision = ""
        self.environment: dict[str, str] = {}
        self.gates: list[dict[str, object]] = []

    def prepare(self, tag: str) -> Path:
        """Select a committed source without touching user edits; example: prepare('v0.2.7')."""
        checkout, self.evidence, self.revision = create_release_checkout(self.root, tag)
        print(f"Martensite pre-release evidence: {self.evidence}", flush=True)
        try:
            self.environment = local_gate_environment(self.root, self.inherited)
            require_cargo_tool_versions(checkout, self.environment)
        except BaseException:
            self.dispose(checkout)
            raise
        return checkout

    def verify(self, checkout: Path) -> None:
        """Run all workflow gate groups in order; example: verify(checkout)."""
        self.gates = verify_workflow_gates(
            checkout, self.source, self.environment, self.evidence
        )
        # Candidate source is frozen; fresh main work cannot silently change the tag target.
        if checked_git(self.root, "rev-parse", "HEAD") != self.revision:
            raise ValueError(
                f"main revision changed: expected tested {self.revision!r}"
            )

    def package(self, checkout: Path, tag: str) -> Path:
        """Run and seal the identical extracted package matrix before tagging; example: package(checkout, tag)."""
        commands = CandidateEnvironmentCommands(
            self.environment, self.evidence / "package-commands"
        )
        verified = build_release_candidate(checkout, tag, commands)
        assets = self.evidence / "artifacts"
        shutil.move(verified, assets)
        validate_assets(assets, tag, self.revision, "Oblikovati/RustyCADKernel")
        return assets

    def receipt(self, checkout: Path, tag: str, assets: Path) -> Path:
        """Bind complete gate and asset evidence to committed source; example: receipt(checkout, tag, assets)."""
        commands = CandidateEnvironmentCommands(
            self.environment, self.evidence / "source-recheck"
        )
        revision = clean_release_revision(checkout, commands)
        if revision != self.revision:
            raise ValueError(
                f"candidate revision={revision!r}: expected {self.revision!r}"
            )
        plan = subprocess.check_output(
            ["git", "show", revision + ":tools/release_verification_plan.py"],
            cwd=checkout,
        )
        path = self.evidence / "acceptance.json"
        payload = self.acceptance_payload(tag, assets, hashlib.sha256(plan).hexdigest())
        path.write_text(json.dumps(payload, indent=2) + "\n")
        return path

    def acceptance_payload(
        self, tag: str, assets: Path, plan_sha256: str
    ) -> dict[str, object]:
        """Keep all required provenance together; example: acceptance_payload(tag, assets, sha)."""
        return {
            "format": 1,
            "tag": tag,
            "revision": self.revision,
            "complete": True,
            "occt_source": str(self.source),
            "verification_plan_sha256": plan_sha256,
            "gates": self.gates,
            "packages": {
                "exit_code": 0,
                "artifacts_path": str(assets.relative_to(self.evidence)),
                "manifest_sha256": hashlib.sha256(
                    (assets / "martensite-release.json").read_bytes()
                ).hexdigest(),
            },
        }

    def tag(self, checkout: Path, tag: str, receipt: Path) -> None:
        """Let the installed hook validate before any release ref exists; example: tag(checkout, tag, receipt)."""
        environment = dict(self.environment, MARTENSITE_RELEASE_RECEIPT=str(receipt))
        subprocess.run(
            [
                "git",
                "tag",
                "-a",
                tag,
                "-m",
                f"Martensite {tag} verified before tagging",
                self.revision,
            ],
            cwd=checkout,
            env=environment,
            check=True,
        )

    def dispose(self, checkout: Path) -> None:
        """Delete source/build artifacts, preserving required receipts; example: dispose(checkout)."""
        try:
            preserve_failure_diagnostics(checkout, self.evidence)
        finally:
            delete_release_checkout(self.root, checkout)
