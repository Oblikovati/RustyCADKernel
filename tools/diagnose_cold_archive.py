#!/usr/bin/env python3
"""Observe the release's first archive test before any GPU warmup.

Example: python3 tools/diagnose_cold_archive.py --root kernel --output evidence --revision SHA
"""

from __future__ import annotations
import argparse
import json
from pathlib import Path
import re
from diagnose_release_renderer import digest_file, record_environment, record_source
from diagnose_source_archives import (
    diagnostic_probe,
    require_clean_source,
    run_renderer_cohort,
)
from renderer_archive_inputs import record_renderer_inputs, require_input_bookend
from renderer_archive_stage import prepare_archive_stage
from diagnose_release_renderer import RendererDiagnosticProcess


def cold_package_arguments(stage: Path) -> list[str]:
    """Preserve the release first-package command; `cold_package_arguments(stage)`."""
    return [
        "cargo",
        "test",
        "--package",
        "martensite-vulkan",
        "--locked",
        "--target-dir",
        str(stage / "target"),
        "--all-targets",
    ]


def retain_cold_executables(probe: RendererDiagnosticProcess, stderr: str) -> None:
    """Hash executed binaries before later diagnostic builds; `retain_cold_executables(probe, stderr)`."""
    records: list[dict[str, str]] = []
    for relative in re.findall(r"Running [^\n]+ \(([^)]+)\)", stderr):
        path = Path(relative)
        path = path if path.is_absolute() else probe.root / path
        if not path.is_file():
            raise ValueError(
                f"received cold executable={path}; expected existing Cargo test binary"
            )
        records.append({"path": str(path), "sha256": digest_file(path)})
    (probe.output / "cold-executed-binaries.json").write_text(
        json.dumps(records, indent=2)
    )


def record_cold_binary_after(probe: RendererDiagnosticProcess) -> None:
    """Retain later-build effects on cold executables; `record_cold_binary_after(probe)`."""
    before = json.loads((probe.output / "cold-executed-binaries.json").read_text())
    after = [
        {"path": item["path"], "sha256": digest_file(Path(item["path"]))}
        for item in before
    ]
    payload = {"before": before, "after": after, "equal": before == after}
    (probe.output / "cold-binary-bookends.json").write_text(
        json.dumps(payload, indent=2)
    )


def run_cold_gate(probe: RendererDiagnosticProcess) -> int:
    """Keep red exits and first-process ordering; `run_cold_gate(probe)`."""
    before = record_renderer_inputs(probe, "cold-inputs-before")
    record_environment(probe)
    result = probe.run("cold-package-all-targets", cold_package_arguments(probe.root))
    retain_cold_executables(probe, result.stderr)
    require_input_bookend(before, record_renderer_inputs(probe, "cold-inputs-after"))
    (probe.output / "cold-verdict.json").write_text(
        json.dumps(
            {
                "revision": probe.revision,
                "exit_code": result.returncode,
                "release_guards_unchanged": True,
                "observation_started_after_gate": True,
            },
            indent=2,
        )
    )
    return result.returncode


def cold_archive_diagnostic(root: Path, output: Path, revision: str) -> None:
    """Stage canonically, test cold, then retain all evidence even when red; `cold_archive_diagnostic(root, out, sha)`."""
    source = diagnostic_probe(root, output / "packaging", revision)
    record_source(source)
    require_clean_source(source, "source-clean-before")
    stage = output / "extracted-workspace"
    prepare_archive_stage(source, stage)
    probe = diagnostic_probe(stage, output / "cold-stage", revision)
    # record_driver/vulkaninfo and examples are deliberately deferred until this exact gate completes.
    run_cold_gate(probe)
    run_renderer_cohort(probe)
    record_cold_binary_after(probe)
    require_clean_source(source, "source-clean-after")


def main() -> None:
    """Collect evidence without promoting a red cold gate to release success; `main()` from the diagnostic workflow."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--revision", required=True)
    args = parser.parse_args()
    cold_archive_diagnostic(args.root.resolve(), args.output.resolve(), args.revision)


if __name__ == "__main__":
    main()
