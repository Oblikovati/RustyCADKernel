"""Bind TKMath runtime evidence to the built OCCT test oracle and libraries."""

from __future__ import annotations

import hashlib
import os
from pathlib import Path
import re
import subprocess

from api_inventory.inputs import JSONObject

NATIVE_BUILD = Path("target/occt-modeling-full")
NATIVE_EXECUTABLE = Path("lin64/gcc/bini/OpenCascadeGTest")


def native_configuration(build: Path, source: Path) -> tuple[Path, Path]:
    """Require a CMake test build tied to OCCT; e.g. `native_configuration(build, source)`."""
    cache = build / "CMakeCache.txt"
    contents = cache.read_text()
    match = re.search(r"^CMAKE_HOME_DIRECTORY:INTERNAL=(.*)$", contents, re.M)
    if match is None or Path(match.group(1)).resolve() != source:
        raise RuntimeError(f"{cache}: expected CMake home directory {source}")
    executable = build / NATIVE_EXECUTABLE
    if not executable.is_file():
        raise RuntimeError(f"{executable}: expected OpenCascadeGTest executable")
    return cache, executable


def sha256(path: Path) -> str:
    """Hash a native provenance input; e.g. `sha256(build / 'CMakeCache.txt')`."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def native_hashes(
    cache: Path, executable: Path, build: Path, source: Path
) -> JSONObject:
    """Fingerprint the test executable and its build libraries; e.g. `native_hashes(cache, exe, build, source)`."""
    return {
        "build_directory": str(build.resolve()),
        "cmake_cache_sha256": sha256(cache),
        "cmake_home_directory": str(source),
        "executable": str(executable.resolve()),
        "executable_sha256": sha256(executable),
        "linked_libraries_sha256": native_library_hashes(executable, build),
    }


def native_library_hashes(executable: Path, build: Path) -> JSONObject:
    """Hash resolved OCCT and GoogleTest libraries; e.g. `native_library_hashes(exe, build)`."""
    result = subprocess.run(
        ["ldd", str(executable)],
        check=True,
        capture_output=True,
        text=True,
        env=native_env(),
    )
    return hash_linked_native_libraries(result.stdout, build)


def hash_linked_native_libraries(output: str, build: Path) -> JSONObject:
    """Hash linked libraries under the native build; e.g. parsed `ldd` output."""
    libraries = linked_native_library_paths(output)
    require_native_library_set(libraries)
    build_root = build.resolve()
    return linked_library_digests(libraries, build_root)


def linked_library_digests(libraries: dict[str, str], build_root: Path) -> JSONObject:
    """Return build-relative hashes; e.g. `linked_library_digests(paths, build)`."""
    return {
        str(Path(path).resolve().relative_to(build_root)): sha256(Path(path))
        for path in sorted(libraries.values())
    }


def linked_native_library_paths(output: str) -> dict[str, str]:
    """Parse OCCT and GoogleTest `ldd` paths; e.g. `libTKMath.so => /build/libTKMath.so`."""
    paths: dict[str, str] = {}
    for line in output.splitlines():
        record_linked_library(line, paths)
    return paths


def record_linked_library(line: str, paths: dict[str, str]) -> None:
    """Record one relevant resolved library or reject missing linkage; e.g. `record_linked_library(line, paths)`."""
    match = re.match(r"\s*(lib(?:TK|gtest)[^ ]*)\s+=>\s+(\S+)", line)
    if match is None:
        return
    if match.group(2) == "not":
        raise RuntimeError(f"Native shared library is unresolved: {match.group(1)}")
    paths[match.group(1)] = match.group(2)


def require_native_library_set(libraries: dict[str, str]) -> None:
    """Require OCCT core and GoogleTest libraries; e.g. TKernel, TKMath, and GTest."""
    prefixes = ("libTKernel.so", "libTKMath.so", "libgtest.so", "libgtest_main.so")
    missing = [
        name for name in prefixes if not any(key.startswith(name) for key in libraries)
    ]
    if missing:
        raise RuntimeError(f"Native runtime lacks linked libraries: {missing!r}")


def verify_native_hashes(
    cache: Path, executable: Path, build: Path, before: JSONObject
) -> None:
    """Reject build or library drift during a run; e.g. `verify_native_hashes(cache, exe, build, before)`."""
    changed = (
        sha256(executable) != before["executable_sha256"]
        or sha256(cache) != before["cmake_cache_sha256"]
        or native_library_hashes(executable, build) != before["linked_libraries_sha256"]
    )
    if changed:
        raise RuntimeError(
            "Native test binary or CMake provenance changed during the run"
        )


def native_env() -> dict[str, str]:
    """Set the native oracle locale and remove loader overrides; e.g. `native_env()`."""
    env = dict(os.environ, LC_ALL="C", LANG="C")
    env.pop("LD_LIBRARY_PATH", None)
    env.pop("LD_PRELOAD", None)
    return env
