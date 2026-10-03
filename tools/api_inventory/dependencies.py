"""Fingerprint development headers so cached ASTs cannot outlive their SDK inputs."""

from __future__ import annotations

import hashlib
import pathlib

from .inputs import JSONObject, digest, toolkit_commands


def header_tree_fingerprint(root: pathlib.Path) -> JSONObject:
    """Bind a configured include tree by path and contents; e.g. `header_tree_fingerprint(Path('/usr/include'))`."""
    accumulator = hashlib.sha256()
    paths = sorted(path for path in root.rglob("*") if path.is_file())
    for path in paths:
        accumulator.update(path.relative_to(root).as_posix().encode())
        accumulator.update(b"\0")
        accumulator.update(digest(path).encode())
        accumulator.update(b"\n")
    return {
        "root": str(root.resolve()),
        "files": len(paths),
        "sha256": accumulator.hexdigest(),
    }


def sdk_fingerprints(llvm_root: pathlib.Path) -> list[JSONObject]:
    """Capture native system and pinned compiler include inputs."""
    roots = [pathlib.Path("/usr/include"), pathlib.Path("/usr/local/include")]
    roots.extend(sorted((llvm_root / "lib/clang").glob("*/include")))
    return [header_tree_fingerprint(root) for root in roots if root.is_dir()]


def compiler_library_fingerprints(llvm_root: pathlib.Path) -> list[JSONObject]:
    """Bind dynamically loaded compiler code; e.g. `compiler_library_fingerprints(llvm_root)`."""
    libraries = [llvm_root / "lib/libclang-cpp.so", llvm_root / "lib/libLLVM.so"]
    return [
        {"path": str(path.resolve()), "sha256": digest(path)}
        for path in libraries
        if path.is_file()
    ]


def external_include_fingerprints(
    database: pathlib.Path, source: pathlib.Path
) -> list[JSONObject]:
    """Hash explicit SDK includes beyond system roots, preserving optional dependencies."""
    roots: set[pathlib.Path] = set()
    for _, flags in toolkit_commands(database, source).values():
        for flag in flags:
            if flag.startswith("-I") and len(flag) > 2:
                candidate = pathlib.Path(flag[2:]).resolve()
                if not any(
                    candidate.is_relative_to(root)
                    for root in (
                        source,
                        database.parent.resolve(),
                        pathlib.Path("/usr/include"),
                    )
                ):
                    roots.add(candidate)
    return [header_tree_fingerprint(root) for root in sorted(roots) if root.is_dir()]
