#!/usr/bin/env python3
"""Build and compare Martensite Vulkan 1.3 with pinned OCCT OpenGL on Linux/X11."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

from vulkan_capture_compare import compare_captures

ROOT = Path(__file__).resolve().parents[1]
PIN = "3d097a0328e71b826377d4814ab05ec3c3d23871"
CASES = [
    (64, 64, 0, 0),
    (96, 48, 0, 2),
    (48, 96, 0, -3),
    (64, 64, 1, 0),
    (96, 48, 1, 2),
    (48, 96, 1, -3),
]


def run(command: list[str], env: dict[str, str]) -> str:
    """Execute a required check without silently skipping; e.g. run(['vulkaninfo','--summary'], env)."""
    result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(
            f"received exit {result.returncode} from {command}:\n{result.stdout}\n{result.stderr}; expected success"
        )
    return result.stdout.strip()


def check_reference(source: Path, build: Path, env: dict[str, str]) -> None:
    """Require the pinned clean source and a matching CMake build; e.g. check_reference(src, build, env)."""
    revision = run(["git", "-C", str(source), "rev-parse", "HEAD"], env)
    dirty = run(
        ["git", "-C", str(source), "status", "--porcelain", "--untracked-files=no"], env
    )
    cache = (build / "CMakeCache.txt").read_text()
    expected = f"CMAKE_HOME_DIRECTORY:INTERNAL={source.resolve()}"
    if revision != PIN or dirty or expected not in cache.splitlines():
        raise ValueError(
            f"received {revision=} {dirty=} build={build}; expected clean {PIN} and {expected}"
        )


def check_shaders(folder: Path, env: dict[str, str]) -> str:
    """Recompile checked-in GLSL for Vulkan 1.3 and compare SPIR-V; e.g. check_shaders(tmp, env)."""
    for stage in ("vert", "frag"):
        source = ROOT / f"crates/vulkan/shaders/unlit.{stage}"
        output = folder / f"unlit.{stage}.spv"
        compile_shader(source, output, env)
        if output.read_bytes() != Path(f"{source}.spv").read_bytes():
            raise ValueError(
                f"received stale SPIR-V for {source}; expected current GLSL compiler output"
            )
    return run(["glslangValidator", "--version"], env)


def compile_shader(source: Path, output: Path, env: dict[str, str]) -> None:
    """Compile a GLSL stage for the required runtime; e.g. compile_shader(vertex, spv, env)."""
    run(
        [
            "glslangValidator",
            "--target-env",
            "vulkan1.3",
            "-V",
            str(source),
            "-o",
            str(output),
        ],
        env,
    )


def build_reference(build: Path, folder: Path, env: dict[str, str]) -> Path:
    """Compile only the test oracle against OCCT; e.g. build_reference(build, tmp, env)."""
    library = build / "lin64/gcc/libi"
    names = "TKOpenGl TKV3d TKService TKHLR TKMesh TKShHealing TKTopAlgo TKGeomAlgo TKBRep TKGeomBase TKG3d TKG2d TKMath TKernel".split()
    output = folder / "opengl-reference"
    run(
        [
            "c++",
            "-std=c++17",
            "-O2",
            f"-I{build / 'include/opencascade'}",
            str(ROOT / "tools/oracle/martensite_vulkan_reference.cxx"),
            *[str(library / f"lib{name}.so") for name in names],
            f"-Wl,-rpath,{library}",
            "-o",
            str(output),
        ],
        env,
    )
    return output


def capture_cases(
    native: Path, rust: Path, folder: Path, env: dict[str, str]
) -> list[dict[str, str | int | float]]:
    """Execute six camera/aspect/translation scenes; e.g. capture_cases(gl, vk, tmp, env)."""
    results = []
    for width, height, perspective, offset in CASES:
        label = f"{width}-{height}-{perspective}-{offset}"
        gl, vk = folder / f"gl-{label}", folder / f"vk-{label}"
        args = list(map(str, (width, height, perspective, offset)))
        native_output = run([str(native), *args, str(gl)], env)
        output = run([str(rust), *args, str(vk)], env)
        row = compare_captures(label, width * height, gl, vk)
        row["runtime_output"] = output
        row["opengl_runtime"] = native_output
        results.append(row)
    return results


def source_hashes() -> dict[str, str]:
    """Fingerprint the actual renderer and transitive source inputs; e.g. source_hashes()."""
    paths = [ROOT / "Cargo.toml", ROOT / "Cargo.lock"]
    for crate in ("vulkan", "tkernel", "tkmath", "tkservice"):
        paths.extend(p for p in (ROOT / "crates" / crate).rglob("*") if p.is_file())
    paths.extend(
        ROOT / p
        for p in (
            "tools/verify_vulkan.py",
            "tools/vulkan_capture_compare.py",
            "tools/oracle/martensite_vulkan_reference.cxx",
        )
    )
    return {
        str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
        for p in sorted(paths)
    }


def verify(source: Path, build: Path, env: dict[str, str]) -> dict[str, object]:
    """Run native and Vulkan acceptance, returning a receipt; e.g. verify(src, build, env)."""
    check_reference(source, build, env)
    target, runtime = build_renderer(env)
    with tempfile.TemporaryDirectory(prefix="martensite-renderer-") as directory:
        folder = Path(directory)
        compiler = check_shaders(folder, env)
        native = build_reference(build, folder, env)
        cases = capture_cases(
            native, target / "debug/examples/graphic3d_capture", folder, env
        )
    return receipt(runtime, compiler, cases, env)


def receipt(
    runtime: str,
    compiler: str,
    cases: list[dict[str, str | int | float]],
    env: dict[str, str],
) -> dict[str, object]:
    """Bind acceptance to concrete sources and devices; e.g. receipt(checks, compiler, cases, env)."""
    return {
        "schema": 1,
        "upstream_revision": PIN,
        "runtime_checks": runtime,
        "cases": cases,
        "shader_compiler": compiler,
        "vulkan_runtime": run(["vulkaninfo", "--summary"], env),
        "source_hashes": source_hashes(),
        "scope": "opaque unlit triangles; default-layer capture; no viewer integration",
    }


def build_renderer(env: dict[str, str]) -> tuple[Path, str]:
    """Compile/test both runtime examples; e.g. build_renderer(env)."""
    target = Path(
        json.loads(run(["cargo", "metadata", "--no-deps", "--format-version=1"], env))[
            "target_directory"
        ]
    )
    run(["cargo", "test", "-p", "martensite-vulkan", "--all-targets", "--locked"], env)
    run(["cargo", "build", "-p", "martensite-vulkan", "--examples", "--locked"], env)
    runtime = run([str(target / "debug/examples/offscreen_acceptance")], env)
    return target, runtime


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--build", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    receipt = verify(args.source.resolve(), args.build.resolve(), dict(os.environ))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(receipt, indent=2) + "\n")
    print(
        f"PASS: 6 OpenGL/Vulkan captures and runtime acceptance; receipt: {args.output}"
    )


if __name__ == "__main__":
    main()
