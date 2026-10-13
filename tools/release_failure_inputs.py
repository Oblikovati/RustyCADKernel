"""Post-failure identity receipts; never initialize a graphics driver before cold tests."""

from __future__ import annotations

import os
import json
from pathlib import Path
import re
import subprocess

from release_archives import file_sha256

ENVIRONMENT_KEYS = (
    "VK_DRIVER_FILES",
    "VK_ICD_FILENAMES",
    "VK_LAYER_PATH",
    "VK_INSTANCE_LAYERS",
    "GALLIUM_OVERRIDE_CPU_CAPS",
    "LP_NATIVE_VECTOR_WIDTH",
    "LIBGL_DRIVERS_PATH",
    "LD_LIBRARY_PATH",
    "RUSTFLAGS",
    "RUST_BACKTRACE",
    "CARGO_BUILD_JOBS",
    "CARGO_PROFILE_TEST_DEBUG",
    "CARGO_INCREMENTAL",
    "CI",
    "RUNNER_OS",
)


def stage_hashes(stage: Path) -> list[dict[str, str | int]]:
    """Hash extracted inputs, excluding build/cache bytes; e.g. stage_hashes(stage)."""
    inputs: list[dict[str, str | int]] = []
    for directory, folders, names in os.walk(stage):
        folders[:] = sorted(
            name
            for name in folders
            if name != ".git" and not (Path(directory) == stage and name == "target")
        )
        for name in sorted(names):
            path = Path(directory) / name
            if path.is_file() and not path.is_symlink():
                inputs.append(file_identity(path, str(path.relative_to(stage))))
    return inputs


def file_identity(path: Path, label: str) -> dict[str, str | int]:
    """Keep only size/hash, never binary contents; e.g. file_identity(path, 'test')."""
    size = path.stat().st_size
    return {"path": label, "size": size, "sha256": file_sha256(path)}


def failed_executables(stage: Path, transcript: str) -> list[Path]:
    """Select actual Cargo-reported processes, not every build; e.g. failed_executables(stage, log)."""
    candidates = re.findall(
        r"process didn't exit successfully: [`']([^`']+)[`']", transcript
    )
    if not candidates:
        running = re.findall(r"Running [^\n]*\(([^()]+)\)", transcript)
        candidates = running[-1:]
    paths: list[Path] = []
    for candidate in candidates:
        path = Path(candidate.split(" --", 1)[0])
        path = path if path.is_absolute() else stage / path
        if path.is_file() and path.resolve().is_relative_to(stage.resolve()):
            paths.append(path)
    return sorted(set(paths))


def library_metadata(executable: Path) -> dict[str, object]:
    """Resolve ELF dependencies only after failure; e.g. library_metadata(test_binary)."""
    result = resolved_library_probe(executable)
    libraries: list[dict[str, str | int]] = []
    for name in re.findall(r"(?:=>\s+)?(/[^\s]+)", result.stdout):
        path = Path(name)
        if path.is_file():
            libraries.append(file_identity(path, str(path)))
    return {
        "tool": "ldd",
        "exit": result.returncode,
        "stdout": result.stdout,
        "stderr": result.stderr,
        "resolved_libraries": libraries,
        "limitation": "Resolved dependencies after exit, not runtime process maps or GPU initialization",
    }


def host_metadata() -> dict[str, object]:
    """Read declared environment and CPU without probing drivers; e.g. host_metadata()."""
    cpu = Path("/proc/cpuinfo")
    return {
        "environment": {
            key: os.environ[key] for key in ENVIRONMENT_KEYS if key in os.environ
        },
        "cpu": cpu.read_text() if cpu.is_file() else "unavailable",
        "platform": (
            os.uname()._asdict() if hasattr(os.uname(), "_asdict") else list(os.uname())
        ),
    }


def failure_inputs(stage: Path, transcript: str) -> dict[str, object]:
    """Capture identities before stage cleanup; e.g. failure_inputs(stage, stderr)."""
    executables: list[dict[str, object]] = []
    for path in failed_executables(stage, transcript):
        executables.append(
            {
                "identity": file_identity(path, str(path.relative_to(stage))),
                "libraries": library_metadata(path),
            }
        )
    return {
        "inputs": stage_hashes(stage),
        "failed_executables": executables,
        "host": host_metadata(),
        "declared_driver_files": declared_driver_files(),
        "missing_executable": not executables,
        "exclusion": "No stage, binary, cache or full archive bytes retained",
    }


def declared_driver_files() -> list[dict[str, object]]:
    """Hash explicitly selected ICD JSON/library without loading it; e.g. declared_driver_files()."""
    declared = os.environ.get("VK_DRIVER_FILES", os.environ.get("VK_ICD_FILENAMES", ""))
    receipts: list[dict[str, object]] = []
    for filename in filter(None, declared.split(os.pathsep)):
        path = Path(filename)
        if path.is_file():
            receipts.append(driver_manifest_identity(path))
    return receipts


def driver_manifest_identity(path: Path) -> dict[str, object]:
    """Only copy declared ICD identity, not arbitrary environment bytes; e.g. driver_manifest_identity(path)."""
    try:
        descriptor = json.loads(path.read_text())
    except (OSError, ValueError) as failure:
        return {"manifest": file_identity(path, str(path)), "error": str(failure)}
    library = Path(descriptor.get("ICD", {}).get("library_path", ""))
    resolved = library if library.is_absolute() else path.parent / library
    receipt: dict[str, object] = {"manifest": file_identity(path, str(path))}
    if resolved.is_file():
        receipt["library"] = file_identity(resolved, str(resolved))
    else:
        receipt["unresolved_library"] = str(library)
    return receipt


def resolved_library_probe(executable: Path) -> subprocess.CompletedProcess[str]:
    """Keep missing inspection tools diagnostic-only; e.g. resolved_library_probe(test)."""
    arguments = ["ldd", str(executable)]
    try:
        return subprocess.run(
            arguments, capture_output=True, text=True, check=False, timeout=15
        )
    except (OSError, subprocess.TimeoutExpired) as failure:
        return subprocess.CompletedProcess(arguments, -1, "", str(failure))
