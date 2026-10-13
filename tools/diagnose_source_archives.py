#!/usr/bin/env python3
"""Compare immutable source and canonical extracted packages on one unchanged runner."""

from __future__ import annotations
import argparse
import json
from pathlib import Path
from diagnose_release_renderer import (
    CAPTURES,
    FILTERS,
    RendererDiagnosticProcess,
    build_artifacts,
    record_driver,
    record_source,
    replay_and_capture,
    record_binary_bookend,
)
from renderer_archive_inputs import (
    record_renderer_inputs,
    require_input_bookend,
    input_differences,
    record_feature_graph,
)
from renderer_archive_stage import prepare_archive_stage

PAIR_FILTERS = (FILTERS[0], FILTERS[3], FILTERS[4])
PAIR_CAPTURES = CAPTURES[:3]


def require_clean_source(probe: RendererDiagnosticProcess, label: str) -> None:
    """Reject mutation of the pinned checkout; `require_clean_source(probe, 'source-clean-before')`."""
    result = probe.run(label, ["git", "status", "--porcelain", "--untracked-files=no"])
    if result.returncode or result.stdout.strip():
        raise ValueError(
            f"received Git exit={result.returncode}, status={result.stdout!r}; expected clean pinned source"
        )
    head = probe.run(f"{label}-head", ["git", "rev-parse", "HEAD"])
    if head.returncode or head.stdout.strip() != probe.revision:
        raise ValueError(
            f"received head={head.stdout.strip()!r}; expected immutable {probe.revision}"
        )


def run_renderer_cohort(probe: RendererDiagnosticProcess) -> dict[str, dict[str, str]]:
    """Preserve strict/full exits and raw buffers from one build root; `run_renderer_cohort(probe)`."""
    record_driver_snapshot(probe, "driver-before")
    before = record_renderer_inputs(probe, "inputs-before")
    record_feature_graph(probe)
    artifacts = build_artifacts(probe, PAIR_CAPTURES)
    toolkit = next(
        artifact.path for artifact in artifacts if artifact.name == "toolkit"
    )
    # The first renderer process keeps the release's default-parallel full-suite context.
    probe.run("full-toolkit", [toolkit, "--nocapture"])
    replay_and_capture(probe, artifacts, PAIR_FILTERS, PAIR_CAPTURES)
    record_binary_bookend(probe, artifacts)
    after = record_renderer_inputs(probe, "inputs-after")
    require_input_bookend(before, after)
    record_driver_snapshot(probe, "driver-after")
    return before


def record_driver_snapshot(probe: RendererDiagnosticProcess, label: str) -> None:
    """Keep CPU/environment/library bookends separate; `record_driver_snapshot(probe, 'driver-before')`."""
    record_driver(probe)
    for name in (
        "libraries-sha256.json",
        "environment.json",
        "cpu.stdout",
        "packages.stdout",
        "vulkan.stdout",
    ):
        path = probe.output / name
        if path.is_file():
            (probe.output / f"{label}-{name}").write_bytes(path.read_bytes())


def paired_renderer_diagnostic(root: Path, output: Path, revision: str) -> None:
    """Keep both cohorts even when strict tests are red; `paired_renderer_diagnostic(root, out, revision)`."""
    source = diagnostic_probe(root, output / "source", revision)
    record_source(source)
    require_clean_source(source, "source-clean-before")
    record_driver(source)
    stage = output / "extracted-workspace"
    prepare_archive_stage(source, stage)
    source_inputs = run_renderer_cohort(source)
    extracted = diagnostic_probe(stage, output / "stage", revision)
    stage_inputs = run_renderer_cohort(extracted)
    require_clean_source(source, "source-clean-after")
    write_pair_comparison(output, revision, source_inputs, stage_inputs)


def diagnostic_probe(
    root: Path, output: Path, revision: str
) -> RendererDiagnosticProcess:
    """Give each build root separate evidence; `diagnostic_probe(root, output, revision)`."""
    output.mkdir(parents=True, exist_ok=True)
    return RendererDiagnosticProcess(root.resolve(), output.resolve(), revision)


def write_pair_comparison(
    output: Path,
    revision: str,
    source: dict[str, dict[str, str]],
    stage: dict[str, dict[str, str]],
) -> None:
    """Compare bytes without normalizing away differences; `write_pair_comparison(out, pin, source, stage)`."""
    payload = {
        "revision": revision,
        "release_guards_unchanged": True,
        "input_differences": {
            kind: input_differences(source[kind], stage[kind]) for kind in source
        },
    }
    (output / "source-stage-comparison.json").write_text(json.dumps(payload, indent=2))


def main() -> None:
    """Run diagnostic evidence rather than release approval; `main()` via the pinned workflow."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--revision", required=True)
    args = parser.parse_args()
    paired_renderer_diagnostic(
        args.root.resolve(), args.output.resolve(), args.revision
    )


if __name__ == "__main__":
    main()
