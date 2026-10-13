"""Create the release's canonical 31-crate stage while retaining every package command."""

from __future__ import annotations
from dataclasses import asdict
import json
from pathlib import Path
from diagnose_release_renderer import RendererDiagnosticProcess, digest_file
from release_archives import stage_workspace
from release_inventory import ReleaseCrate, release_inventory
from release_packages import package_test_support


class SourceArchiveCommands:
    def __init__(self, probe: RendererDiagnosticProcess) -> None:
        self.probe = probe
        self.sequence = 0

    def run(self, arguments: list[str], root: Path) -> str:
        """Preserve canonical package calls and failures; `commands.run(args, source_root)`."""
        if root != self.probe.root:
            raise ValueError(
                f"received command root={root}; expected pinned root={self.probe.root}"
            )
        self.sequence += 1
        result = self.probe.run(f"package-command-{self.sequence:03}", arguments)
        if result.returncode:
            raise ValueError(
                f"received package exit={result.returncode}, command={arguments}; expected exit=0"
            )
        return result.stdout


def prepare_archive_stage(
    probe: RendererDiagnosticProcess, stage: Path
) -> list[ReleaseCrate]:
    """Use canonical packaging and test-support extraction; `prepare_archive_stage(source_probe, stage)`."""
    commands = SourceArchiveCommands(probe)
    crates = release_inventory(probe.root, commands)
    if len(crates) != 31:
        raise ValueError(
            f"received package count={len(crates)}; expected exactly 31 Martensite packages"
        )
    commands.run(["python3", "tools/toolkit_test_suites.py"], probe.root)
    commands.run(
        ["cargo", "package", "--workspace", "--no-verify", "--exclude-lockfile"],
        probe.root,
    )
    package_test_support(probe.root, commands)
    archives = probe.root / "target/package"
    record_archive_provenance(probe, archives, crates)
    stage_workspace(archives, stage, crates, probe.root / "Cargo.lock")
    return crates


def record_archive_provenance(
    probe: RendererDiagnosticProcess, archives: Path, crates: list[ReleaseCrate]
) -> None:
    """Retain exact archive bytes and inventory; `record_archive_provenance(probe, archives, crates)`."""
    names = [crate.archive for crate in crates] + ["martensite-test-support.tar.gz"]
    payload = {
        "revision": probe.revision,
        "inventory": [asdict(crate) for crate in crates],
        "archives": {name: digest_file(archives / name) for name in names},
    }
    (probe.output / "archive-provenance.json").write_text(json.dumps(payload, indent=2))
