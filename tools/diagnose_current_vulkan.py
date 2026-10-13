#!/usr/bin/env python3
"""Mirror the current Vulkan CI gate before installing observation-only prerequisites.

Example: python3 tools/diagnose_current_vulkan.py --root kernel --output evidence --revision SHA --phase gate
"""

from __future__ import annotations
import argparse
import json
from pathlib import Path
from diagnose_cold_archive import record_cold_binary_after, run_cold_gate
from diagnose_release_renderer import build_artifacts, record_source, replay_and_capture
from diagnose_source_archives import (
    PAIR_CAPTURES,
    PAIR_FILTERS,
    diagnostic_probe,
    record_driver_snapshot,
    require_clean_source,
)
from renderer_archive_inputs import (
    record_feature_graph,
    record_renderer_inputs,
    require_input_bookend,
)
from diagnose_release_renderer import RendererDiagnosticProcess


def current_vulkan_arguments() -> list[str]:
    """Preserve the immutable CI invocation exactly; `current_vulkan_arguments()`."""
    return ["cargo", "test", "-p", "martensite-vulkan", "--all-targets", "--locked"]


def current_vulkan_gate(probe: RendererDiagnosticProcess) -> None:
    """Retain the original cold verdict before any graphics observation; `current_vulkan_gate(probe)`."""
    record_source(probe)
    require_clean_source(probe, "gate-source-before")
    run_cold_gate(probe, current_vulkan_arguments())
    require_clean_source(probe, "gate-source-after")


def require_current_gate(probe: RendererDiagnosticProcess) -> None:
    """Reject observation without its pinned original gate; `require_current_gate(probe)`."""
    path = probe.output / "cold-verdict.json"
    if not path.is_file():
        raise ValueError(
            f"received verdict={path}; expected completed original current Vulkan gate"
        )
    verdict = json.loads(path.read_text())
    if verdict["revision"] != probe.revision:
        raise ValueError(
            f"received revision={verdict['revision']}; expected immutable {probe.revision}"
        )


def current_vulkan_observations(probe: RendererDiagnosticProcess) -> None:
    """Collect raw buffers after the original verdict, without replacing it; `current_vulkan_observations(probe)`."""
    require_current_gate(probe)
    require_clean_source(probe, "observed-source-before")
    record_driver_snapshot(probe, "driver-before")
    before = record_renderer_inputs(probe, "inputs-before")
    record_feature_graph(probe)
    artifacts = build_artifacts(probe, PAIR_CAPTURES)
    replay_and_capture(probe, artifacts, PAIR_FILTERS, PAIR_CAPTURES)
    require_input_bookend(before, record_renderer_inputs(probe, "inputs-after"))
    record_cold_binary_after(probe)
    record_driver_snapshot(probe, "driver-after")
    require_clean_source(probe, "observed-source-after")


def main() -> None:
    """Run separate gate and observation phases; `main()` from the exact-mirror workflow."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--revision", required=True)
    parser.add_argument("--phase", choices=("gate", "observe"), required=True)
    args = parser.parse_args()
    probe = diagnostic_probe(args.root, args.output, args.revision)
    action = (
        current_vulkan_gate if args.phase == "gate" else current_vulkan_observations
    )
    action(probe)


if __name__ == "__main__":
    main()
