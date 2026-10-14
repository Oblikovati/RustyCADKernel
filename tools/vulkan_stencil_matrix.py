"""Run the complete controlled stencil corpus, preserving raw strict failures; e.g. --help."""

from __future__ import annotations
import argparse
from dataclasses import dataclass
import json
import os
from pathlib import Path
import subprocess
from typing import Protocol
from vulkan_capture_compare import compare_bytes
from vulkan_stencil_attachment import validate_attachment
from vulkan_ordered_stencil_cases import OrderedStencilCase, ordered_stencil_cases


@dataclass(frozen=True)
class StencilPrograms:
    root: Path
    native: Path
    rust: Path
    mesa: Path
    validation: Path
    output: Path


class StencilCaptureIO(Protocol):
    def run(self, argv: list[str], environment: dict[str, str], log: Path) -> int: ...
    def read(self, path: Path) -> bytes: ...


class SystemStencilCaptureIO:
    def run(self, argv: list[str], environment: dict[str, str], log: Path) -> int:
        """Preserve complete process output; e.g. io.run(argv, env, log)."""
        with log.open("wb") as stream:
            result = subprocess.run(
                argv, env=environment, stdout=stream, stderr=subprocess.STDOUT
            )
        return result.returncode

    def read(self, path: Path) -> bytes:
        """Read immutable captured bytes; e.g. io.read(prefix.with_suffix('.rgba'))."""
        with path.open("rb") as stream:
            return stream.read()


def stencil_environment(
    programs: StencilPrograms, profile: str, inherited: dict[str, str]
) -> dict[str, str]:
    """Pin genuine25.2.8 userspace and explicit hardware; e.g. stencil_environment(paths,'host',env)."""
    env = _clean_stencil_environment(inherited)
    library = programs.mesa / "usr/lib/x86_64-linux-gnu"
    icd = _stencil_icd(profile)
    env.update(
        DISPLAY=env.get("DISPLAY", ":1"),
        LD_LIBRARY_PATH=str(library),
        LIBGL_DRIVERS_PATH=str(library / "dri"),
        LIBGL_ALWAYS_SOFTWARE="1" if profile == "software" else "0",
        VK_DRIVER_FILES=str(programs.mesa / f"usr/share/vulkan/icd.d/{icd}_icd.json"),
        MARTENSITE_VULKAN_DEVICE="0",
        GALLIUM_OVERRIDE_CPU_CAPS="sse2",
        LP_NATIVE_VECTOR_WIDTH="128",
    )
    return env


def _clean_stencil_environment(inherited: dict[str, str]) -> dict[str, str]:
    """Remove inherited driver/version substitutions; e.g. _clean_stencil_environment(env)."""
    removed = {
        "VK_ICD_FILENAMES",
        "GALLIUM_DRIVER",
        "MESA_LOADER_DRIVER_OVERRIDE",
        "LLVM_TARGET_FEATURES",
        "MESA_GL_VERSION_OVERRIDE",
        "MESA_GLSL_VERSION_OVERRIDE",
    }
    return {key: value for key, value in inherited.items() if key not in removed}


def _stencil_icd(profile: str) -> str:
    """Require an explicit tested profile; e.g. _stencil_icd('host')."""
    if profile not in ("software", "host"):
        raise ValueError(f"profile={profile!r}; expected software or host")
    return "lvp" if profile == "software" else "radeon"


def stencil_command(
    programs: StencilPrograms, renderer: str, case: OrderedStencilCase, prefix: Path
) -> list[str]:
    """Use actual native/Rust executables; e.g. stencil_command(paths,'native',case,prefix)."""
    args = [case.mode, case.method, str(prefix), str(case.seed)]
    if renderer == "native":
        return [str(programs.native), *args]
    return [
        "python3",
        str(programs.root / "tools/with_vulkan_validation.py"),
        "--cache",
        str(programs.validation),
        "--",
        str(programs.rust),
        *args,
    ]


def stencil_prefix(
    programs: StencilPrograms,
    profile: str,
    case: OrderedStencilCase,
    renderer: str,
    repeat: int,
) -> Path:
    """Bind each output to its full domain; e.g. stencil_prefix(paths,'host',case,'native',1)."""
    name = f"{profile}-{case.name}-{renderer}-{repeat}"
    return programs.output / name


def check_repeat(
    io: StencilCaptureIO, first: Path, second: Path, suffixes: tuple[str, ...]
) -> None:
    """Require byte-identical independent runs; e.g. check_repeat(io,a,b,('rgba','depth'))."""
    for suffix in suffixes:
        left, right = Path(f"{first}.{suffix}"), Path(f"{second}.{suffix}")
        if io.read(left) != io.read(right):
            raise ValueError(
                f"repeat bytes differ: {left} versus {right}; expected identical {suffix}"
            )


def compare_stencil_case(
    io: StencilCaptureIO, native: Path, rust: Path, label: str
) -> float:
    """Preserve exactRGBA/finite1e-6depth/state gates; e.g. compare_stencil_case(io,a,b,'enable')."""
    buffers = [
        io.read(Path(f"{prefix}.{suffix}"))
        for suffix in ("rgba", "depth")
        for prefix in (native, rust)
    ]
    result = compare_bytes(label, 4096, *buffers)
    native_state, rust_state = (
        io.read(Path(f"{prefix}.stencil-state")) for prefix in (native, rust)
    )
    if native_state != rust_state:
        raise ValueError(
            f"state {native_state!r} != {rust_state!r}; expected exact native context state"
        )
    return result.maximum_depth_delta


def validate_stencil_identities(
    io: StencilCaptureIO, native: Path, rust: Path, profile: str
) -> None:
    """Reject substituted cohorts/devices; e.g. validate_stencil_identities(io,a,b,'software')."""
    native_log, rust_log = (
        io.read(Path(f"{prefix}.log")).decode() for prefix in (native, rust)
    )
    for log in (native_log, rust_log):
        _validate_stencil_identity(log, profile)


def _validate_stencil_identity(log: str, profile: str) -> None:
    """Require the actually attributed cohort/device; e.g. _validate_stencil_identity(log,'host')."""
    if "Mesa 25.2.8" not in log:
        raise ValueError(f"identity={log!r}; expected actual Mesa25.2.8")
    software = "llvmpipe" in log
    if software != (profile == "software") or (profile == "host" and "AMD" not in log):
        raise ValueError(
            f"profile={profile} identity={log!r}; expected explicit llvmpipe/AMD device"
        )


def stencil_case_result(
    programs: StencilPrograms,
    profile: str,
    case: OrderedStencilCase,
    io: StencilCaptureIO,
) -> dict[str, object]:
    """Preserve strict results without discarding failures; e.g. stencil_case_result(paths,'host',case,io)."""
    native, rust = (
        stencil_prefix(programs, profile, case, renderer, 1)
        for renderer in ("native", "rust")
    )
    _check_case_repetitions(programs, profile, case, io)
    validate_stencil_storage(io, native)
    validate_stencil_identities(io, native, rust, profile)
    maximum = compare_stencil_case(io, native, rust, case.name)
    return {"complete": True, "maximum_depth_delta": maximum}


def validate_stencil_storage(io: StencilCaptureIO, native: Path) -> None:
    """Check every observed real attachment; e.g. validate_stencil_storage(io, prefix)."""
    rows = io.read(Path(f"{native}.stencil-trace")).decode().splitlines()
    if not rows:
        raise ValueError(
            f"trace={native}; expected nonempty actual draw attachment records"
        )
    for line in rows:
        validate_attachment(json.loads(line))


def _check_case_repetitions(
    programs: StencilPrograms,
    profile: str,
    case: OrderedStencilCase,
    io: StencilCaptureIO,
) -> None:
    """Check each backend's independent repeat; e.g. _check_case_repetitions(paths,'host',case,io)."""
    for renderer in ("native", "rust"):
        first, second = (
            stencil_prefix(programs, profile, case, renderer, repeat)
            for repeat in (1, 2)
        )
        suffixes = ("rgba", "depth", "stencil-state") + (
            ("stencil-trace", "draw-trace") if renderer == "native" else ()
        )
        check_repeat(io, first, second, suffixes)


def execute_stencil_case(
    programs: StencilPrograms,
    profile: str,
    case: OrderedStencilCase,
    env: dict[str, str],
    io: StencilCaptureIO,
) -> dict[str, object]:
    """Execute all four runs before strict review; e.g. execute_stencil_case(paths,'host',case,env,io)."""
    runs = _stencil_capture_runs(programs, profile, case, env, io)
    result: dict[str, object] = dict(
        profile=profile, case=case.name, runs=runs, complete=False
    )
    if any(run["exit_code"] != 0 for run in runs):
        return result
    try:
        result.update(stencil_case_result(programs, profile, case, io))
    except (ValueError, OSError) as error:
        result["strict_failure"] = str(error)
    return result


def _stencil_capture_runs(
    programs: StencilPrograms,
    profile: str,
    case: OrderedStencilCase,
    env: dict[str, str],
    io: StencilCaptureIO,
) -> list[dict[str, object]]:
    """Preserve every command/terminal status; e.g. _stencil_capture_runs(paths,'host',case,env,io)."""
    runs = []
    for renderer, repeat in (("native", 1), ("native", 2), ("rust", 1), ("rust", 2)):
        prefix = stencil_prefix(programs, profile, case, renderer, repeat)
        argv = stencil_command(programs, renderer, case, prefix)
        runs.append(
            {"argv": argv, "exit_code": io.run(argv, env, Path(f"{prefix}.log"))}
        )
    return runs


def matrix_arguments() -> StencilPrograms:
    """Resolve explicit executable/runtime ownership; e.g. python3 tools/vulkan_stencil_matrix.py --help."""
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("root", "native", "rust", "mesa", "validation", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    paths = StencilPrograms(
        *(
            getattr(args, name).resolve()
            for name in ("root", "native", "rust", "mesa", "validation", "output")
        )
    )
    paths.output.mkdir(parents=True, exist_ok=True)
    return paths


def main() -> None:
    """Capture87cases twice/profile with no timing; e.g. python3 tools/vulkan_stencil_matrix.py --help."""
    programs = matrix_arguments()
    with (programs.output / "results.ndjson").open("w") as stream:
        for profile in ("software", "host"):
            env = stencil_environment(programs, profile, dict(os.environ))
            for case in ordered_stencil_cases():
                result = execute_stencil_case(
                    programs, profile, case, env, SystemStencilCaptureIO()
                )
                line = json.dumps(result)
                stream.write(line + "\n")
                stream.flush()
                print(line, flush=True)


if __name__ == "__main__":
    main()
