"""One Martensite local/CI verification contract; e.g. verification_gates(root, source)."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

GATE_NAMES = (
    "rust-prechecks",
    "rust-tests",
    "rust-docs",
    "coverage",
    "vulkan-cold",
    "vulkan-acceptance",
    "vulkan-shaders",
    "vulkan-performance",
    "dependencies",
    "inventory",
    "tools",
)


# Local and hosted verification use identical artifact/CPU policy; VVL paths stay inherited.
VERIFICATION_ENV: tuple[tuple[str, str], ...] = (
    ("CARGO_PROFILE_DEV_DEBUG", "0"),
    ("CARGO_PROFILE_TEST_DEBUG", "0"),
    ("CARGO_INCREMENTAL", "0"),
    ("GALLIUM_OVERRIDE_CPU_CAPS", "sse2"),
    ("LP_NATIVE_VECTOR_WIDTH", "128"),
    ("LLVM_PROFILE_FILE_NAME", "martensite-%4m.profraw"),
)


@dataclass(frozen=True)
class GateCommand:
    """Exact argument vector and command-local overrides, without a shell."""

    argv: tuple[str, ...]
    env: tuple[tuple[str, str], ...] = ()
    stdout_path: Path | None = None


@dataclass(frozen=True)
class VerificationGate:
    """Ordered commands forming one mandatory gate."""

    name: str
    commands: tuple[GateCommand, ...]


def verification_gates(root: Path, occt_source: Path) -> tuple[VerificationGate, ...]:
    """Return every workflow gate; e.g. verification_gates(Path.cwd(), Path('../OCCT'))."""
    gates = (*rust_gates(), coverage_gate(), *vulkan_gates(root))
    return (*gates, *terminal_gates(occt_source))


def terminal_gates(occt_source: Path) -> tuple[VerificationGate, ...]:
    """Keep dependencies, inventory and tools mandatory; e.g. terminal_gates(source)."""
    inventory = (
        GateCommand(
            ("python3", "tools/porting.py", "check", "--source", str(occt_source))
        ),
        GateCommand(("python3", "tools/porting.py", "status")),
    )
    return (
        VerificationGate("dependencies", (GateCommand(("cargo", "deny", "check")),)),
        VerificationGate("inventory", inventory),
        VerificationGate(
            "tools",
            (GateCommand(("python3", "tools/run_tool_tests.py", "--jobs", "2")),),
        ),
    )


def rust_gates() -> tuple[VerificationGate, ...]:
    """Preserve discovery/formatter/lint before tests; e.g. rust_gates()[0].name."""
    prechecks = (
        GateCommand(("python3", "tools/toolkit_test_suites.py")),
        GateCommand(("python3", "tools/check_rust_format.py")),
        GateCommand(("cargo", "lint")),
    )
    tests = GateCommand(("cargo", "test-all", "--no-fail-fast"))
    return (
        VerificationGate("rust-prechecks", prechecks),
        VerificationGate("rust-tests", (tests,)),
        rust_docs_gate(),
    )


def rust_docs_gate() -> VerificationGate:
    """Warnings remain documentation failures; e.g. rust_docs_gate().commands[0].env."""
    argv = ("cargo", "doc", "--workspace", "--all-features", "--no-deps", "--locked")
    command = GateCommand(argv, (("RUSTDOCFLAGS", "-D warnings"),))
    return VerificationGate("rust-docs", (command,))


COVERAGE_ARGV = (
    "cargo",
    "llvm-cov",
    "nextest",
    "--workspace",
    "--all-features",
    "--locked",
    "--profile",
    "ci",
    "-E",
    "not test(=bvh_bug30655::BVH_PairDistanceTest__Bug30655_SphereDistance)",
    "--lcov",
    "--output-path",
    "target/lcov.info",
)


def coverage_gate() -> VerificationGate:
    """Keep the original dense replay exclusion; e.g. coverage_gate().commands[0].argv."""
    command = GateCommand(COVERAGE_ARGV)
    return VerificationGate("coverage", (command,))


def xvfb_example(name: str, release: bool = False) -> tuple[str, ...]:
    """Keep the real graphical harness; e.g. xvfb_example('lighting_acceptance')."""
    profile = ("--release",) if release else ()
    return (
        "xvfb-run",
        "-a",
        "-s",
        "-screen 0 1280x1024x24",
        "cargo",
        "run",
        "-p",
        "martensite-vulkan",
        *profile,
        "--example",
        name,
        "--locked",
    )


def vulkan_gates(root: Path) -> tuple[VerificationGate, ...]:
    """Cold tests precede acceptance and timing; e.g. vulkan_gates(root)[0].name."""
    cold = GateCommand(
        ("cargo", "test", "-p", "martensite-vulkan", "--all-targets", "--locked")
    )
    acceptance = (
        GateCommand(xvfb_example("offscreen_acceptance")),
        GateCommand(xvfb_example("lighting_acceptance")),
    )
    return (
        VerificationGate("vulkan-cold", (cold,)),
        VerificationGate("vulkan-acceptance", acceptance),
        VerificationGate("vulkan-shaders", shader_commands(root)),
        vulkan_performance_gate(),
    )


def vulkan_performance_gate() -> VerificationGate:
    """Preserve actual performance stdout; e.g. vulkan_performance_gate().commands."""
    command = GateCommand(
        xvfb_example("renderer_performance", True),
        stdout_path=Path("target/renderer-performance.log"),
    )
    return VerificationGate("vulkan-performance", (command,))


def shader_commands(root: Path) -> tuple[GateCommand, ...]:
    """Validate every committed variant and original controls; e.g. shader_commands(root)."""
    variants = tuple(sorted((root / "crates/vulkan/shaders").glob("*.spv")))
    if not variants:
        raise ValueError(
            f"shader variants={variants!r}; expected nonempty crates/vulkan/shaders/*.spv"
        )
    commands = tuple(
        GateCommand(
            (
                "spirv-val",
                "--target-env",
                "vulkan1.3",
                str(path.relative_to(root)),
            )
        )
        for path in variants
    )
    return (*commands, *shader_controls())


def shader_controls() -> tuple[GateCommand, ...]:
    """Retain vertex/fragment compilation controls; e.g. shader_controls()[0].argv."""
    commands: list[GateCommand] = []
    for stage in ("vert", "frag"):
        commands.extend(shader_stage_controls(stage))
    return tuple(commands)


def shader_stage_controls(stage: str) -> tuple[GateCommand, GateCommand]:
    """Compile and validate the unchanged control; e.g. shader_stage_controls('vert')."""
    source = f"crates/vulkan/shaders/unlit.{stage}"
    compile_argv = (
        "glslangValidator",
        "--target-env",
        "vulkan1.3",
        "-V",
        source,
        "-o",
        f"/tmp/unlit.{stage}.spv",
    )
    validate = ("spirv-val", "--target-env", "vulkan1.3", source + ".spv")
    return GateCommand(compile_argv), GateCommand(validate)
