#!/usr/bin/env python3
"""Mirror the current Vulkan CI gate before installing observation-only prerequisites.

Example: python3 tools/diagnose_current_vulkan.py --root kernel --output evidence --revision SHA --phase gate
"""

from __future__ import annotations
import argparse
import json
from pathlib import Path
from diagnose_cold_archive import (
    record_cold_binary_after,
    run_cold_gate,
    retain_cold_executables,
)
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
from diagnose_release_renderer import RendererDiagnosticProcess, DiagnosticCommandResult


def current_vulkan_arguments() -> list[str]:
    """Preserve the immutable CI invocation exactly; `current_vulkan_arguments()`."""
    return ["cargo", "test", "-p", "martensite-vulkan", "--all-targets", "--locked"]


def gate_profile_arguments(profile: str) -> list[str]:
    """Keep original aliases distinct from controlled builds; `gate_profile_arguments('workspace')`."""
    if profile == "workspace":
        return ["cargo", "test-all", "--no-fail-fast"]
    if profile == "vulkan":
        return current_vulkan_arguments()
    raise ValueError(f"gate profile={profile!r}; expected workspace or vulkan")


def current_vulkan_gate(probe: RendererDiagnosticProcess) -> None:
    """Retain the original cold verdict before any graphics observation; `current_vulkan_gate(probe)`."""
    record_source(probe)
    require_clean_source(probe, "gate-source-before")
    run_cold_gate(probe, current_vulkan_arguments())
    require_clean_source(probe, "gate-source-after")


def record_existing_gate(
    probe: RendererDiagnosticProcess,
    gate_log: Path,
    gate_exit: Path,
    profile: str = "vulkan",
) -> None:
    """Import an already completed original gate without executing it; `record_existing_gate(probe, log, status)`."""
    contents = gate_log.read_text()
    validate_original_log(probe, contents, profile)
    status = gate_exit.read_text().strip()
    if not status.isdecimal() or not 0 <= int(status) <= 255:
        raise ValueError(
            f"gate exit={status!r}; expected an integer status from 0 through 255"
        )
    (probe.output / "original-gate.log").write_text(contents)
    write_imported_verdict(probe, int(status), profile)
    # Save the cold verdict before optional metadata can fail; never replace its status.
    record_source(probe)
    require_clean_source(probe, "imported-source")
    retain_cold_executables(probe, contents)


def validate_original_log(
    probe: RendererDiagnosticProcess, contents: str, profile: str = "vulkan"
) -> None:
    """Bind imported evidence to the original source and command; `validate_original_log(probe, log)`."""
    expected = [
        f"Martensite original gate revision: {probe.revision}",
        "Martensite original gate command: "
        + " ".join(gate_profile_arguments(profile)),
    ]
    received = contents.splitlines()[:2]
    if received != expected:
        raise ValueError(
            f"gate log headers={received!r}; expected original identity {expected!r}"
        )


def write_imported_verdict(
    probe: RendererDiagnosticProcess, status: int, profile: str = "vulkan"
) -> None:
    """Keep the original exit independently of observation; `write_imported_verdict(probe, 101)`."""
    verdict = {
        "revision": probe.revision,
        "exit_code": status,
        "arguments": gate_profile_arguments(profile),
        "profile": profile,
        "release_guards_unchanged": True,
        "observation_started_after_gate": True,
        "imported_original_gate": True,
    }
    (probe.output / "cold-verdict.json").write_text(json.dumps(verdict, indent=2))


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
    parser.add_argument("--phase", choices=("gate", "observe", "record"), required=True)
    parser.add_argument("--gate-log", type=Path)
    parser.add_argument("--gate-exit", type=Path)
    parser.add_argument("--profile", choices=("vulkan", "workspace"), default="vulkan")
    args = parser.parse_args()
    probe = diagnostic_probe(args.root, args.output, args.revision)
    restore_command_history(probe)
    dispatch_phase(probe, args, parser)


def restore_command_history(probe: RendererDiagnosticProcess) -> None:
    """Append phase observations without deleting earlier evidence; `restore_command_history(probe)`."""
    path = probe.output / "commands.json"
    if not path.is_file():
        return
    recorded = json.loads(path.read_text())
    if recorded.get("revision") != probe.revision or not isinstance(
        recorded.get("commands"), list
    ):
        raise ValueError(
            f"command history={recorded!r}; expected revision {probe.revision} and commands list"
        )
    probe.results.extend(recorded_command(entry) for entry in recorded["commands"])


def recorded_command(entry: object) -> DiagnosticCommandResult:
    """Require actual serialized command records; `recorded_command(receipt)`."""
    keys = {"label", "arguments", "exit_code", "elapsed_seconds"}
    if not isinstance(entry, dict) or set(entry) != keys:
        raise ValueError(
            f"command record={entry!r}; expected exact fields {sorted(keys)}"
        )
    arguments = entry["arguments"]
    if not isinstance(arguments, list) or not all(
        isinstance(arg, str) for arg in arguments
    ):
        raise ValueError(
            f"command arguments={arguments!r}; expected string argument list"
        )
    validate_command_scalars(entry)
    return DiagnosticCommandResult(
        entry["label"], arguments, entry["exit_code"], entry["elapsed_seconds"]
    )


def validate_command_scalars(entry: dict[str, object]) -> None:
    """Reject recast or invalid status/timing evidence; `validate_command_scalars(receipt)`."""
    if not isinstance(entry["label"], str) or type(entry["exit_code"]) is not int:
        raise ValueError(
            f"command record={entry!r}; expected string label and integer exit"
        )
    if (
        type(entry["elapsed_seconds"]) not in (int, float)
        or entry["elapsed_seconds"] < 0
    ):
        raise ValueError(
            f"command elapsed={entry['elapsed_seconds']!r}; expected nonnegative seconds"
        )


def dispatch_phase(
    probe: RendererDiagnosticProcess,
    args: argparse.Namespace,
    parser: argparse.ArgumentParser,
) -> None:
    """Require original gate inputs for record mode; `dispatch_phase(probe, arguments, parser)`."""
    if args.phase == "record":
        if args.gate_log is None or args.gate_exit is None:
            parser.error("record phase requires --gate-log and --gate-exit paths")
        record_existing_gate(
            probe, args.gate_log, args.gate_exit, getattr(args, "profile", "vulkan")
        )
        return
    if getattr(args, "profile", "vulkan") == "workspace":
        dispatch_workspace_phase(probe, args.phase, parser)
        return
    action = (
        current_vulkan_gate if args.phase == "gate" else current_vulkan_observations
    )
    action(probe)


def dispatch_workspace_phase(
    probe: RendererDiagnosticProcess, phase: str, parser: argparse.ArgumentParser
) -> None:
    """Require the completed original workspace gate; `dispatch_workspace_phase(probe, 'observe', parser)`."""
    if phase != "observe":
        parser.error(
            "workspace profile supports original record and observe phases only"
        )
    from renderer_workspace_evidence import workspace_observations

    workspace_observations(probe)


if __name__ == "__main__":
    main()
