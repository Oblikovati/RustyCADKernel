#!/usr/bin/env python3
"""Capture native and Rust torus rows without accepting or normalizing differences.

Example: python3 tools/ci/capture_platform_torus.py
"""

from pathlib import Path
import hashlib
import json
import os
import platform
import subprocess
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from test_target_selection import cargo_case_command


def capture_native(root: Path, output: Path) -> Path:
    """Compile the pinned native probe; capture_native(root, output)."""
    build = root / "target/native-platform"
    libraries = next(build.rglob("libTKBO.dylib")).parent
    names = "TKBO TKPrim TKShHealing TKTopAlgo TKGeomAlgo TKBRep TKGeomBase TKG3d TKG2d TKMath TKernel".split()
    executable = output / "native-torus"
    command = [
        "c++",
        "-std=c++17",
        "-O2",
        *contraction_flags(),
        f"-I{build / 'include/opencascade'}",
        str(root / "tools/oracle/tkbo_bop_torus.cxx"),
        *[str(libraries / f"lib{name}.dylib") for name in names],
        f"-Wl,-rpath,{libraries}",
        "-o",
        str(executable),
    ]
    subprocess.run(command, cwd=root, check=True)
    rows = subprocess.check_output([str(executable)], cwd=root)
    (output / "native.expected").write_bytes(rows)
    return executable


def contraction_flags() -> list[str]:
    """Match the native library's diagnostic FP policy; contraction_flags()."""
    policy = os.environ.get("MARTENSITE_FP_CONTRACT")
    if policy is None:
        return []
    if policy not in ("off", "on", "fast"):
        raise ValueError(f"FP contraction {policy!r}: expected off, on, or fast")
    return [f"-ffp-contract={policy}"]


def capture_rust(root: Path, output: Path) -> int:
    """Add only a diagnostic print, run the original assertions, and restore; capture_rust(root, output)."""
    source = root / "crates/tkbo/tests/bop_algo_bop_torus.rs"
    original = source.read_text()
    marker = "    let expected = if cfg!"
    if original.count(marker) != 1:
        raise ValueError(
            f"Marker count {original.count(marker)}: expected exactly one fixture comparison"
        )
    source.write_text(
        original.replace(
            marker,
            '    println!("MARTENSITE_TORUS_BEGIN\\n{rows}MARTENSITE_TORUS_END");\n'
            + marker,
        )
    )
    try:
        with (output / "rust-test.log").open("w") as log:
            result = subprocess.run(
                cargo_case_command(root, 'rustycad-tkbo', 'bop_algo_bop_torus', ['--nocapture']),
                cwd=root,
                stdout=log,
                stderr=subprocess.STDOUT,
            )
        return result.returncode
    finally:
        source.write_text(original)


def main() -> None:
    """Record platform provenance and preserve both verdicts; main()."""
    root = Path(__file__).resolve().parents[2]
    output = root / "target/platform-torus"
    output.mkdir(parents=True, exist_ok=True)
    executable = capture_native(root, output)
    status = capture_rust(root, output)
    receipt = {
        "platform": platform.platform(),
        "machine": platform.machine(),
        "probe_fp_flags": contraction_flags(),
        "rust_test_exit": status,
        "native_sha256": hashlib.sha256(
            (output / "native.expected").read_bytes()
        ).hexdigest(),
        "source_revision": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=root, text=True
        ).strip(),
        "occt_revision": subprocess.check_output(
            ["git", "-C", "target/occt-source", "rev-parse", "HEAD"],
            cwd=root,
            text=True,
        ).strip(),
    }
    (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    executable.unlink()
    raise SystemExit(status)


if __name__ == "__main__":
    main()
