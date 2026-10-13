"""Replay original workspace failures and capture with their all-feature graph."""

from __future__ import annotations

from dataclasses import asdict
import json
from pathlib import Path
import re

from diagnose_cold_archive import record_cold_binary_after
from diagnose_current_vulkan import require_current_gate
from diagnose_release_renderer import (
    DiagnosticBinary,
    RendererDiagnosticProcess,
    cargo_executables,
    digest_file,
    replay_strict_filters,
    replay_raw_captures,
    record_buffer_hashes,
    record_binary_bookend,
)
from diagnose_source_archives import (
    PAIR_CAPTURES,
    PAIR_FILTERS,
    record_driver_snapshot,
    require_clean_source,
)
from renderer_archive_inputs import record_renderer_inputs, require_input_bookend

ALL_FEATURE_FLAGS: tuple[str, ...] = ("--all-features", "--locked")


class PackageReplayProbe(RendererDiagnosticProcess):
    """Lend Cargo's package cwd while sharing the authoritative command history."""

    def __init__(self, original: RendererDiagnosticProcess) -> None:
        super().__init__(
            original.root / "crates/vulkan", original.output, original.revision
        )
        # Both transports append to one history; a separate list would overwrite commands.json.
        self.results = original.results
        context = {
            "cwd": str(self.root),
            "filters": list(PAIR_FILTERS),
            "reason": "Cargo integration tests execute in their package manifest directory",
        }
        (self.output / "original-replay-context.json").write_text(
            json.dumps(context, indent=2)
        )


def original_workspace_toolkit(probe: RendererDiagnosticProcess) -> DiagnosticBinary:
    """Select the executed Vulkan process, never a stale glob; `original_workspace_toolkit(probe)`."""
    contents = (probe.output / "original-gate.log").read_text()
    matches = list(re.finditer(r"Running [^\n]+ \(([^)]+)\)", contents))
    selected: list[Path] = []
    for index, match in enumerate(matches):
        end = matches[index + 1].start() if index + 1 < len(matches) else len(contents)
        if any(name in contents[match.end() : end] for name in PAIR_FILTERS):
            path = Path(match[1])
            selected.append(path if path.is_absolute() else probe.root / path)
    if len(selected) != 1:
        raise ValueError(
            f"original Vulkan binaries={selected!r}; expected exactly one executed toolkit"
        )
    verify_original_binary(probe, selected[0])
    return DiagnosticBinary("toolkit", str(selected[0]), digest_file(selected[0]))


def verify_original_binary(probe: RendererDiagnosticProcess, path: Path) -> None:
    """Require the imported binary bytes before replay; `verify_original_binary(probe, path)`."""
    records = json.loads((probe.output / "cold-executed-binaries.json").read_text())
    expected = [record["sha256"] for record in records if record["path"] == str(path)]
    if expected != [digest_file(path)]:
        raise ValueError(
            f"original binary={str(path)!r}, hashes={expected!r}; expected unchanged executed bytes"
        )


def workspace_build_arguments(probe: RendererDiagnosticProcess) -> list[str]:
    """Preserve workspace selection and keep cold binaries untouched; `workspace_build_arguments(probe)`."""
    arguments = [
        "cargo",
        "build",
        "--workspace",
        "--all-features",
        "--locked",
        "--message-format=json",
    ]
    arguments += ["--target-dir", str(probe.output / "workspace-build")]
    for example, _, _ in PAIR_CAPTURES:
        arguments += ["--example", example]
    return arguments


def workspace_capture_artifacts(
    probe: RendererDiagnosticProcess,
) -> list[DiagnosticBinary]:
    """Select genuine Vulkan examples from workspace compiler messages; `workspace_capture_artifacts(probe)`."""
    result = probe.run("workspace-build-captures", workspace_build_arguments(probe))
    if result.returncode:
        raise ValueError(
            f"workspace capture build exit={result.returncode}; expected successful all-feature build"
        )
    artifacts = vulkan_workspace_executables(probe, result.stdout)
    expected = {name for name, _, _ in PAIR_CAPTURES}
    if {artifact.name for artifact in artifacts} != expected or len(artifacts) != len(
        expected
    ):
        raise ValueError(
            f"workspace examples={[item.name for item in artifacts]!r}; expected exact {sorted(expected)!r}"
        )
    return artifacts


def vulkan_workspace_executables(
    probe: RendererDiagnosticProcess, output: str
) -> list[DiagnosticBinary]:
    """Avoid same-name targets from other packages; `vulkan_workspace_executables(probe, json_lines)`."""
    selected: list[str] = []
    manifest = str(probe.root / "crates/vulkan/Cargo.toml")
    for line in output.splitlines():
        try:
            record = json.loads(line)
        except json.JSONDecodeError:
            continue
        if (
            record.get("reason") == "compiler-artifact"
            and record.get("manifest_path") == manifest
        ):
            selected.append(line)
    return cargo_executables("\n".join(selected))


def workspace_feature_graph(probe: RendererDiagnosticProcess) -> None:
    """Record the actual selected graph, not single-package all-features; `workspace_feature_graph(probe)`."""
    metadata = ["cargo", "metadata", *ALL_FEATURE_FLAGS, "--format-version=1"]
    tree = ["cargo", "tree", "--workspace", *ALL_FEATURE_FLAGS, "-e", "features"]
    commands = [
        ("workspace-feature-tree", tree),
        ("workspace-feature-metadata", metadata),
    ]
    for label, arguments in commands:
        result = probe.run(label, arguments)
        if result.returncode:
            raise ValueError(
                f"{label} exit={result.returncode}; expected completed workspace feature graph"
            )


def workspace_replay(probe: RendererDiagnosticProcess) -> None:
    """Use the original failing process for strict controls; `workspace_replay(probe)`."""
    original = original_workspace_toolkit(probe)
    replay_strict_filters(PackageReplayProbe(probe), original.path, PAIR_FILTERS)
    artifacts = [original, *workspace_capture_artifacts(probe)]
    (probe.output / "binaries-before.json").write_text(
        json.dumps([asdict(item) for item in artifacts], indent=2)
    )
    replay_raw_captures(
        probe, {item.name: item.path for item in artifacts}, PAIR_CAPTURES
    )
    record_binary_bookend(probe, artifacts)
    record_buffer_hashes(probe)
    record_cold_binary_after(probe)


def workspace_observations(probe: RendererDiagnosticProcess) -> None:
    """Observe workspace failures without default-feature substitution; `workspace_observations(probe)`."""
    require_current_gate(probe)
    verdict = json.loads((probe.output / "cold-verdict.json").read_text())
    if verdict.get("profile") != "workspace":
        raise ValueError(f'gate profile={verdict.get("profile")!r}; expected workspace')
    require_clean_source(probe, "workspace-observed-before")
    record_driver_snapshot(probe, "driver-before")
    before = record_renderer_inputs(probe, "inputs-before")
    workspace_feature_graph(probe)
    workspace_replay(probe)
    require_input_bookend(before, record_renderer_inputs(probe, "inputs-after"))
    record_driver_snapshot(probe, "driver-after")
    require_clean_source(probe, "workspace-observed-after")
