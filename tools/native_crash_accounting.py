"""Reusable accounting distinguishes exact parity from native-crash divergences."""

from __future__ import annotations
import hashlib
import subprocess
from pathlib import Path
from verify_tkmath_numeric import run_probe, compare


def replay_crash(native: Path, rust: Path, request: str) -> dict[str, str | int]:
    """Require native SIGSEGV and the documented Rust failure; replay_crash(native,rust,request)."""
    original = subprocess.run(
        [str(native)], input=request + "\n", text=True, capture_output=True, timeout=30
    )
    translated = subprocess.run(
        [str(rust)], input=request + "\n", text=True, capture_output=True, timeout=30
    )
    validate_crash(original.returncode, translated.returncode, translated.stdout)
    return {
        "input": request,
        "input_sha256": hashlib.sha256(request.encode()).hexdigest(),
        "native_returncode": original.returncode,
        "rust_returncode": translated.returncode,
        "rust_exception": "Standard_ConstructionError",
        "rust_output": translated.stdout,
        "classification": "native_crash_rust_typed_failure_divergence",
    }


def validate_crash(native_code: int, rust_code: int, rust_output: str) -> None:
    """Reject changed native behavior or a Rust crash; validate_crash(-11,0,'Standard_ConstructionError')."""
    if native_code != -11:
        raise ValueError(
            f"Native return code {native_code}; expected -11 SIGSEGV, reclassify the divergence"
        )
    if rust_code != 0 or "Standard_ConstructionError" not in rust_output.split():
        raise ValueError(
            f"Rust return code {rust_code}, output {rust_output!r}; expected 0 with Standard_ConstructionError"
        )


def verify_defined(native: Path, rust: Path, requests: list[str], output: Path) -> None:
    """Compare the defined paths exactly; verify_defined(native,rust,requests,output)."""
    inputs = "\n".join(requests) + "\n"
    (output / "inputs.txt").write_text(inputs)
    original = run_probe([str(native)], output / "native.txt", inputs)
    translated = run_probe([str(rust)], output / "rust.txt", inputs)
    compare(original, translated, inputs)


def partition_crash_requests(
    candidates: list[str], manifest: frozenset[str]
) -> tuple[list[str], list[str]]:
    """Keep all requests and reject missing declared crashes; partition_crash_requests(rows,hashes)."""
    exact: list[str] = []
    crashes: list[str] = []
    found: set[str] = set()
    for request in candidates:
        digest = hashlib.sha256(request.encode()).hexdigest()
        if digest in manifest:
            crashes.append(request)
            found.add(digest)
        else:
            exact.append(request)
    if found != manifest:
        raise ValueError(f"Native crash inventory {found}; expected {manifest}")
    return exact, crashes
