#!/usr/bin/env python3
"""Build and compare Martensite Vulkan 1.3 with pinned OCCT OpenGL on Linux/X11."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import subprocess
import tempfile

from dataclasses import asdict, dataclass
from vulkan_build import build_vulkan_examples
from vulkan_capture_artifacts import FileCaptureStore, export_comparison
from vulkan_profiles import VulkanProfile, probe_profile, confirm_device
from vulkan_performance import (
    performance_rows,
    regression_failures,
    startup_milliseconds,
)

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
AIS_MODES = (
    "ais-displayed",
    "ais-erased",
    "ais-redisplayed",
    "ais-mode1",
    "ais-affinity-off",
    "ais-cleared",
    "ais-removed",
    "ais-recreated",
    "ais-erased-all",
    "ais-displayed-all",
    "ais-removed-all",
    "ais-erased-selected",
    "ais-displayed-selected",
)


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


@dataclass(frozen=True)
class HarnessConfig:
    captures: Path
    profiles: tuple[str, ...] = ("lavapipe", "host")
    host_icd: Path | None = None
    samples: int = 100
    warmup: int = 20
    baseline: dict[str, object] | None = None
    median_ratio: float = 1.25
    p95_ratio: float = 1.5

    def __post_init__(self) -> None:
        if not 20 <= self.samples <= 10000 or not 1 <= self.warmup <= 10000:
            raise ValueError(
                f"received samples={self.samples} warmup={self.warmup}; expected samples 20..10000 and warmup 1..10000"
            )
        if any(
            not math.isfinite(ratio) or ratio < 1
            for ratio in (self.median_ratio, self.p95_ratio)
        ):
            raise ValueError(
                f"received ratios {(self.median_ratio, self.p95_ratio)}; expected finite limits >= 1"
            )


def capture_case(
    native: Path,
    rust: Path,
    folder: Path,
    profile: VulkanProfile,
    case: tuple[int, int, int, int],
    mode: str,
) -> dict[str, object]:
    """Capture actual V3d output on the selected device; e.g. capture_case(gl, vk, dir, profile, case, mode)."""
    width, height, perspective, offset = case
    label = f"{width}-{height}-{perspective}-{offset}-{mode}"
    gl, vk = folder / f"gl-{label}", folder / f"vk-{label}"
    clear_previous_case(gl, vk)
    args = list(map(str, case))
    native_output = run([str(native), *args, str(gl), mode], profile.environment)
    executable = rust.with_name("ais_capture") if mode.startswith("ais-") else rust
    output = run([str(executable), *args, str(vk), mode], profile.environment)
    confirm_device(output, profile.device)
    software_gl = any(
        name in native_output.lower()
        for name in ("llvmpipe", "softpipe", "software rasterizer")
    )
    if software_gl != (profile.name == "lavapipe"):
        raise ValueError(
            f"received OpenGL device {native_output}; expected {profile.name} graphics"
        )
    row = export_comparison(label, width, height, gl, vk, FileCaptureStore())
    row.update(runtime_output=output, opengl_runtime=native_output)
    return row


def clear_previous_case(native: Path, ported: Path) -> None:
    """Remove only this generated case's stale artifacts; e.g. clear_previous_case(gl, vk)."""
    for prefix in (native, ported):
        for suffix in (
            "rgba",
            "depth",
            "png",
            "depth.png",
            "diff.png",
            "depth-diff.png",
        ):
            Path(f"{prefix}.{suffix}").unlink(missing_ok=True)


def capture_cases(
    native: Path, rust: Path, folder: Path, profile: VulkanProfile
) -> list[dict[str, object]]:
    """Retain raw and AIS lifecycle scenes and continue collecting failures; e.g. capture_cases(gl, vk, folder, profile)."""
    folder.mkdir(parents=True, exist_ok=True)
    results = []
    for case, mode in [
        (case, mode) for case in CASES for mode in ("default", "full", *AIS_MODES)
    ]:
        try:
            results.append(capture_case(native, rust, folder, profile, case, mode))
        except (RuntimeError, ValueError) as error:
            results.append(
                {"case": [*case, mode], "status": "failed", "error": str(error)}
            )
    return results


def source_hashes() -> dict[str, str]:
    """Fingerprint the actual renderer and transitive source inputs; e.g. source_hashes()."""
    paths = [ROOT / "Cargo.toml", ROOT / "Cargo.lock"]
    for crate in sorted(p.name for p in (ROOT / "crates").iterdir() if p.is_dir()):
        paths.extend(p for p in (ROOT / "crates" / crate).rglob("*") if p.is_file())
    paths.extend(
        ROOT / p
        for p in (
            "tools/verify_vulkan.py",
            "tools/vulkan_capture_compare.py",
            "tools/oracle/martensite_vulkan_reference.cxx",
            "tools/oracle/martensite_vulkan_ais.cxx",
        )
    )
    paths.extend(ROOT.glob("tools/vulkan_*.py"))
    paths.extend(ROOT.glob("tools/tests/test_vulkan_*.py"))
    return {
        str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
        for p in sorted(paths)
    }


def verify_profile(
    profile: VulkanProfile, target: Path, native: Path, config: HarnessConfig
) -> dict[str, object]:
    """Run functional and performance gates on one pinned device; e.g. verify_profile(profile, target, gl, config)."""
    runtime = run(
        [str(target / "debug/examples/offscreen_acceptance")], profile.environment
    )
    confirm_device(runtime, profile.device)
    cases = capture_cases(
        native,
        target / "debug/examples/graphic3d_capture",
        config.captures / profile.name,
        profile,
    )
    output = measure_profile(profile, target, config)
    result = profile_result(profile, runtime, cases, output)
    return finish_profile(result, config)


def finish_profile(
    result: dict[str, object], config: HarnessConfig
) -> dict[str, object]:
    """Apply the timing gate and mark the complete profile; e.g. finish_profile(result, config)."""
    failures = check_baseline(result, config)
    result.update(
        performance_failures=failures,
        performance_baseline="compared" if config.baseline else "collected",
        status=(
            "failed"
            if failures or any(row["status"] != "passed" for row in result["cases"])
            else "passed"
        ),
    )
    return result


def measure_profile(profile: VulkanProfile, target: Path, config: HarnessConfig) -> str:
    """Run optimized timings on the verified device; e.g. measure_profile(profile, target, config)."""
    output = run(
        [
            str(target / "release/examples/renderer_performance"),
            str(config.samples),
            str(config.warmup),
        ],
        profile.environment,
    )
    confirm_device(output, profile.device)
    return output


def profile_result(
    profile: VulkanProfile, runtime: str, cases: list[dict[str, object]], output: str
) -> dict[str, object]:
    """Describe functional and timing evidence; e.g. profile_result(profile, runtime, cases, output)."""
    return {
        "profile": profile.name,
        "device": asdict(profile.device),
        "driver_info": profile.probe,
        "runtime_checks": runtime,
        "cases": cases,
        "performance": performance_rows(output),
        "startup_ms": startup_milliseconds(output),
        "timing_scope": "release V3d redraw, scene extraction, Vulkan allocation/submission, fence wait and color/depth readback; validation disabled",
    }


def check_baseline(result: dict[str, object], config: HarnessConfig) -> list[str]:
    """Bind regression limits to a matching recorded profile; e.g. check_baseline(result, config)."""
    if config.baseline is None:
        return []
    profiles = config.baseline.get("profiles", [])
    previous = next(
        (row for row in profiles if row.get("profile") == result["profile"]), None
    )
    if previous is None or previous.get("status") != "passed":
        return [
            f"received no successful baseline for {result['profile']}; expected a passed profile"
        ]
    result["baseline_timings"] = [
        {key: row[key] for key in ("workload", "median_ms", "p95_ms")}
        for row in previous["performance"]
    ]
    return regression_failures(result, previous, config.median_ratio, config.p95_ratio)


def profile_matrix(
    target: Path, native: Path, config: HarnessConfig, env: dict[str, str]
) -> list[dict[str, object]]:
    """Require every requested profile; unavailable hardware is a failure; e.g. profile_matrix(target, gl, config, env)."""
    results = []
    manifests = sorted(Path("/usr/share/vulkan/icd.d").glob("*.json"))
    for name in config.profiles:
        try:
            profile = probe_profile(name, env, manifests, run, config.host_icd)
            results.append(verify_profile(profile, target, native, config))
        except (RuntimeError, ValueError) as error:
            results.append({"profile": name, "status": "failed", "error": str(error)})
    return results


def verify(
    source: Path, build: Path, env: dict[str, str], config: HarnessConfig
) -> dict[str, object]:
    """Run the functional/performance matrix and keep captures; e.g. verify(src, build, env, config)."""
    check_reference(source, build, env)
    target, tests = build_vulkan_examples(ROOT, env, run)
    with tempfile.TemporaryDirectory(prefix="martensite-renderer-build-") as directory:
        folder = Path(directory)
        compiler = check_shaders(folder, env)
        native = build_reference(build, folder, env)
        profiles = profile_matrix(target, native, config, env)
    return receipt(tests, compiler, profiles, env, config)


def receipt(
    tests: str,
    compiler: str,
    profiles: list[dict[str, object]],
    env: dict[str, str],
    config: HarnessConfig,
) -> dict[str, object]:
    """Bind the completed matrix to its source inputs; e.g. receipt(tests, compiler, profiles, env, config)."""
    return {
        "schema": 2,
        "upstream_revision": PIN,
        "martensite_revision": run(["git", "rev-parse", "HEAD"], env),
        "status": (
            "passed" if all(row["status"] == "passed" for row in profiles) else "failed"
        ),
        "cpu_tests": tests,
        "profiles": profiles,
        "shader_compiler": compiler,
        "source_hashes": source_hashes(),
        "performance_limits": {
            "median_ratio": config.median_ratio,
            "p95_ratio": config.p95_ratio,
        },
        "scope": "opaque unlit triangles through V3d/Graphic3d/AIS; default/full-layer and AIS lifecycle captures; end-to-end frame benchmarks",
    }


def parse_arguments(argv: list[str] | None = None) -> argparse.Namespace:
    """Parse explicit profiles and regression budgets; e.g. parse_arguments()."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--build", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument(
        "--captures",
        type=Path,
        help="persistent color/depth PNG, raw and diff directory",
    )
    parser.add_argument(
        "--profiles",
        nargs="+",
        choices=("lavapipe", "host"),
        default=["lavapipe", "host"],
    )
    parser.add_argument("--host-icd", type=Path)
    performance_arguments(parser)
    return parser.parse_args(argv)


def performance_arguments(parser: argparse.ArgumentParser) -> None:
    """Expose matching workload and budget controls; e.g. performance_arguments(parser)."""
    parser.add_argument("--samples", type=int, default=100)
    parser.add_argument("--warmup", type=int, default=20)
    parser.add_argument("--performance-baseline", type=Path)
    parser.add_argument("--median-ratio", type=float, default=1.25)
    parser.add_argument("--p95-ratio", type=float, default=1.5)


def main() -> None:
    args = parse_arguments()
    config = configuration(args)
    result = verify(
        args.source.resolve(), args.build.resolve(), dict(os.environ), config
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(
        f"{result['status'].upper()}: renderer functional/performance matrix; receipt: {args.output}; captures: {config.captures}"
    )
    if result["status"] != "passed":
        raise SystemExit(1)


def configuration(args: argparse.Namespace) -> HarnessConfig:
    """Resolve output paths and optional baseline; e.g. configuration(args)."""
    return HarnessConfig(
        (args.captures or args.output.with_suffix(".captures")).resolve(),
        tuple(dict.fromkeys(args.profiles)),
        args.host_icd,
        args.samples,
        args.warmup,
        (
            json.loads(args.performance_baseline.read_text())
            if args.performance_baseline
            else None
        ),
        args.median_ratio,
        args.p95_ratio,
    )


if __name__ == "__main__":
    main()
