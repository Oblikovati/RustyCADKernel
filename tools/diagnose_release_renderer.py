#!/usr/bin/env python3
"""Capture immutable Martensite release failures without changing any release gate or asset."""

from __future__ import annotations
import argparse
from dataclasses import asdict, dataclass
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time
from renderer_process_maps import run_observed_command

REVISION = "11a4c51420695911ea94d9b4be23c11df21a0d55"
FILTERS = (
    "textured_native_mapping_assets_replay_on_either_actual_device",
    "fixed_lighting_native_assets_replay_without_occt",
    "native_fixed_filter_mipmap_and_alpha_assets_replay_without_occt",
    "scoped_native_edge_and_mapped_line_assets_replay_without_occt",
    "xr_real_presentations_replay_native_captures_on_each_device",
)
CAPTURES = (
    ("ais_textured_shape_capture", "map-5", "map-5"),
    ("xr_device_capture", "texture", "xr-texture"),
    ("triangle_edges_capture", "feather-two", "feather-two"),
    (
        "fixed_lighting_capture",
        "lit-alpha-opacity-updated",
        "lit-alpha-opacity-updated",
    ),
    (
        "capping_texture_capture",
        "cap-section-edges-generated-polygon-thin-fixed-bilinear-fixed-center-edges-disabled",
        "cap-fixed-bilinear-center",
    ),
)


DIAGNOSTIC_ENVIRONMENT_KEYS = (
    "VK_DRIVER_FILES",
    "VK_LAYER_PATH",
    "LD_LIBRARY_PATH",
    "LD_PRELOAD",
    "VK_LOADER_DEBUG",
    "GALLIUM_OVERRIDE_CPU_CAPS",
    "LP_NATIVE_VECTOR_WIDTH",
    "LP_NUM_THREADS",
    "GALLIVM_PERF",
    "LLVM_TARGET_FEATURES",
    "DISPLAY",
    "MARTENSITE_VULKAN_DEVICE",
    "RUST_TEST_THREADS",
    "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "RUSTUP_TOOLCHAIN",
    "CARGO_INCREMENTAL",
    "CARGO_PROFILE_DEV_DEBUG",
    "CARGO_PROFILE_TEST_DEBUG",
    "CARGO_PROFILE_DEV_OPT_LEVEL",
    "CARGO_PROFILE_TEST_OPT_LEVEL",
    "CARGO_BUILD_JOBS",
    "MESA_SHADER_CACHE_DISABLE",
    "MESA_SHADER_CACHE_DIR",
)


@dataclass(frozen=True)
class DiagnosticCommandResult:
    label: str
    arguments: list[str]
    exit_code: int
    elapsed_seconds: float


@dataclass(frozen=True)
class DiagnosticBinary:
    name: str
    path: str
    sha256: str


class RendererDiagnosticProcess:
    def __init__(self, root: Path, output: Path, revision: str = REVISION) -> None:
        self.revision = revision
        self.root = root
        self.output = output
        self.results: list[DiagnosticCommandResult] = []

    def run(self, label: str, arguments: list[str]) -> subprocess.CompletedProcess[str]:
        """Retain stdout/stderr and real exit status; `probe.run('cpu', ['lscpu'])`."""
        started = time.monotonic()
        result = run_observed_command(arguments, self.root, self.output / label)
        (self.output / f"{label}.stdout").write_text(result.stdout)
        (self.output / f"{label}.stderr").write_text(result.stderr)
        self.results.append(
            DiagnosticCommandResult(
                label, arguments, result.returncode, time.monotonic() - started
            )
        )
        self.persist()
        return result

    def persist(self) -> None:
        """Retain every completed command even before later failures; `probe.persist()`."""
        payload = {
            "revision": self.revision,
            "release_guards_unchanged": True,
            "commands": [asdict(result) for result in self.results],
        }
        (self.output / "commands.json").write_text(json.dumps(payload, indent=2))


def digest_file(path: Path) -> str:
    """Identify exact source/binary bytes; `digest_file(Path('Cargo.lock'))`."""
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def cargo_executables(output: str) -> list[DiagnosticBinary]:
    """Select actual Cargo artifacts, avoiding stale glob matches; `cargo_executables(json_lines)`."""
    artifacts: list[DiagnosticBinary] = []
    for line in output.splitlines():
        try:
            entry = json.loads(line)
        except json.JSONDecodeError:
            continue
        if entry.get("reason") != "compiler-artifact" or not entry.get("executable"):
            continue
        path = Path(entry["executable"])
        artifacts.append(
            DiagnosticBinary(entry["target"]["name"], str(path), digest_file(path))
        )
    return artifacts


def record_source(probe: RendererDiagnosticProcess) -> None:
    """Prove immutable checkout and compiled inputs; `record_source(probe)`."""
    revision = probe.run("source-head", ["git", "rev-parse", "HEAD"])
    if revision.stdout.strip() != probe.revision:
        raise ValueError(
            f"received revision={revision.stdout.strip()!r}; expected immutable {probe.revision}"
        )
    probe.run("source-status", ["git", "status", "--porcelain"])
    files = probe.run("source-files", ["git", "ls-files", "-z"])
    hashes = {
        name: digest_file(probe.root / name)
        for name in files.stdout.split("\0")
        if name and (probe.root / name).is_file()
    }
    (probe.output / "source-sha256.json").write_text(json.dumps(hashes, sort_keys=True))


def record_driver(probe: RendererDiagnosticProcess) -> None:
    """Record CPU, packages, loader and actually selected ICD; `record_driver(probe)`."""
    for label, arguments in (
        ("cpu", ["lscpu"]),
        ("kernel", ["uname", "-a"]),
        (
            "packages",
            ["dpkg-query", "-W", "mesa-vulkan-drivers", "libllvm*", "libvulkan1"],
        ),
        ("vulkan", ["env", "VK_LOADER_DEBUG=driver", "vulkaninfo", "--summary"]),
        ("toolchain", ["rustup", "show"]),
    ):
        probe.run(label, arguments)
    icd = Path(os.environ["VK_DRIVER_FILES"])
    library = Path(json.loads(icd.read_text())["ICD"]["library_path"])
    library = resolve_driver_library(probe, icd, library)
    record_libraries(probe, icd, library)


def resolve_driver_library(
    probe: RendererDiagnosticProcess, icd: Path, library: Path
) -> Path:
    """Resolve ICD sonames through loader search paths; `resolve_driver_library(probe, icd, soname)`."""
    if library.is_absolute() or "/" in str(library):
        return library if library.is_absolute() else icd.parent / library
    for folder in os.environ.get("LD_LIBRARY_PATH", "").split(":"):
        candidate = Path(folder) / library
        if folder and candidate.is_file():
            return candidate
    linked = probe.run("loader-cache", ["ldconfig", "-p"])
    for line in linked.stdout.splitlines():
        if line.strip().startswith(f"{library} ") and "=>" in line:
            candidate = Path(line.split("=>", 1)[1].strip())
            if candidate.is_file():
                return candidate
    raise ValueError(
        f"received ICD library={library}; expected resolvable driver soname or file"
    )


def record_libraries(
    probe: RendererDiagnosticProcess, icd: Path, library: Path
) -> None:
    """Hash actual linked driver dependencies and pinned layer; `record_libraries(probe, icd, library)`."""
    linked = probe.run("driver-linked", ["ldd", str(library)]).stdout
    paths = [icd, library]
    paths.extend(linked_library_paths(linked))
    layer = Path(os.environ["VK_LAYER_PATH"])
    paths.extend(layer.glob("*.json"))
    paths.extend(Path(os.environ["LD_LIBRARY_PATH"].split(":")[0]).glob("*.so*"))
    hashes = {
        str(path.resolve()): digest_file(path) for path in paths if path.is_file()
    }
    (probe.output / "libraries-sha256.json").write_text(
        json.dumps(hashes, sort_keys=True, indent=2)
    )
    record_environment(probe)


def linked_library_paths(linked: str) -> list[Path]:
    """Select actual absolute ldd dependencies; `linked_library_paths(ldd_stdout)`."""
    return [
        Path(line.split("=>", 1)[1].strip().split()[0])
        for line in linked.splitlines()
        if "=> /" in line
    ]


def record_environment(probe: RendererDiagnosticProcess) -> None:
    """Capture only rendering selection variables; `record_environment(probe)`."""
    selected = {name: os.environ.get(name) for name in DIAGNOSTIC_ENVIRONMENT_KEYS}
    (probe.output / "environment.json").write_text(json.dumps(selected, indent=2))


def build_artifacts(
    probe: RendererDiagnosticProcess,
    captures: tuple[tuple[str, str, str], ...] = CAPTURES,
) -> list[DiagnosticBinary]:
    """Build only immutable test/capture targets; `build_artifacts(probe)`."""
    common = ["-p", "martensite-vulkan", "--locked", "--message-format=json"]
    test = probe.run(
        "build-toolkit", ["cargo", "test", *common, "--test", "toolkit", "--no-run"]
    )
    captured = build_capture_targets(probe, common, captures)
    if test.returncode or captured.returncode:
        raise ValueError(
            f"received Cargo exits={test.returncode},{captured.returncode}; expected successful immutable builds"
        )
    artifacts = cargo_executables(test.stdout) + cargo_executables(captured.stdout)
    (probe.output / "binaries-before.json").write_text(
        json.dumps([asdict(item) for item in artifacts], indent=2)
    )
    return artifacts


def build_capture_targets(
    probe: RendererDiagnosticProcess,
    common: list[str],
    captures: tuple[tuple[str, str, str], ...],
) -> subprocess.CompletedProcess[str]:
    """Build exactly the requested raw capture targets; `build_capture_targets(probe, common, captures)`."""
    arguments = ["cargo", "build", *common]
    for example, _, _ in captures:
        arguments.extend(["--example", example])
    return probe.run("build-captures", arguments)


def replay_and_capture(
    probe: RendererDiagnosticProcess,
    artifacts: list[DiagnosticBinary],
    filters: tuple[str, ...] = FILTERS,
    captures: tuple[tuple[str, str, str], ...] = CAPTURES,
) -> None:
    """Keep strict red tests separate from successful raw captures; `replay_and_capture(probe, artifacts)`."""
    binaries = {artifact.name: artifact.path for artifact in artifacts}
    if "toolkit" not in binaries:
        raise ValueError(
            f"received targets={sorted(binaries)}; expected compiled toolkit"
        )
    replay_strict_filters(probe, binaries["toolkit"], filters)
    replay_raw_captures(probe, binaries, captures)
    record_binary_bookend(probe, artifacts)
    record_buffer_hashes(probe)


def replay_raw_captures(
    probe: RendererDiagnosticProcess,
    binaries: dict[str, str],
    captures: tuple[tuple[str, str, str], ...],
) -> None:
    """Preserve actual raw captures even after red replays; `replay_raw_captures(probe, binaries, captures)`."""
    for example, mode, label in captures:
        if example not in binaries:
            raise ValueError(
                f"received targets={sorted(binaries)}; expected capture target {example}"
            )
        probe.run(
            f"capture-{label}", [binaries[example], mode, str(probe.output / label)]
        )


def record_buffer_hashes(probe: RendererDiagnosticProcess) -> None:
    """Identify uploaded raw buffers; `record_buffer_hashes(probe)`."""
    hashes = {
        str(path.name): digest_file(path)
        for path in probe.output.iterdir()
        if path.suffix in (".rgba", ".depth")
    }
    (probe.output / "buffers-sha256.json").write_text(
        json.dumps(hashes, sort_keys=True, indent=2)
    )


def replay_strict_filters(
    probe: RendererDiagnosticProcess, binary: str, filters: tuple[str, ...] = FILTERS
) -> None:
    """Retain every real strict-test verdict; `replay_strict_filters(probe, toolkit)`."""
    for name in filters:
        probe.run(f"strict-{name}", [binary, name, "--nocapture"])


def record_binary_bookend(
    probe: RendererDiagnosticProcess, artifacts: list[DiagnosticBinary]
) -> None:
    """Prove captures used unmodified binaries; `record_binary_bookend(probe, artifacts)`."""
    after = [
        DiagnosticBinary(item.name, item.path, digest_file(Path(item.path)))
        for item in artifacts
    ]
    (probe.output / "binaries-after.json").write_text(
        json.dumps([asdict(item) for item in after], indent=2)
    )
    require_binary_bookend(artifacts, after)


def require_binary_bookend(
    before: list[DiagnosticBinary], after: list[DiagnosticBinary]
) -> None:
    """Reject a changed executable after preserving evidence; `require_binary_bookend(before, after)`."""
    if before != after:
        changed = [
            item.path
            for item, fresh in zip(before, after, strict=True)
            if item != fresh
        ]
        raise ValueError(
            f"received changed diagnostic binaries={changed}; expected unchanged capture/replay executables"
        )


def main() -> None:
    """Run diagnostic evidence, not release approval; `main()` via the explicit workflow."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--revision", default=REVISION)
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    probe = RendererDiagnosticProcess(args.root.resolve(), output, args.revision)
    record_source(probe)
    record_driver(probe)
    replay_and_capture(probe, build_artifacts(probe))
    probe.run("source-status-after", ["git", "status", "--porcelain"])


if __name__ == "__main__":
    main()
