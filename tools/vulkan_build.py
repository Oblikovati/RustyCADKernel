"""Own a per-checkout Cargo cache so shared worktrees cannot substitute binaries."""

from collections.abc import Callable
from pathlib import Path

CargoRunner = Callable[[list[str], dict[str, str]], str]
PACKAGE = "martensite-vulkan"


def vulkan_build_environment(
    workspace: Path, environment: dict[str, str]
) -> dict[str, str]:
    """Isolate outputs without mutating caller settings; vulkan_build_environment(root, env)."""
    # Cargo can report a fresh example while its shared, unhashed alias belongs to another checkout.
    # Keeping the cache beneath this checkout also makes worktree deletion reclaim its artifacts.
    return {**environment, "CARGO_TARGET_DIR": str(workspace / "target/vulkan-harness")}


def build_vulkan_examples(
    workspace: Path, environment: dict[str, str], runner: CargoRunner
) -> tuple[Path, str]:
    """Build functional and release timing executables in one owned cache; build_vulkan_examples(root, env, run)."""
    isolated = vulkan_build_environment(workspace, environment)
    tests = runner(
        ["cargo", "test", "-p", PACKAGE, "--all-targets", "--locked"], isolated
    )
    runner(["cargo", "build", "-p", PACKAGE, "--examples", "--locked"], isolated)
    build_vulkan_performance(isolated, runner)
    return Path(isolated["CARGO_TARGET_DIR"]), tests


def build_vulkan_performance(environment: dict[str, str], runner: CargoRunner) -> None:
    """Require production optimization in the same isolated cache; build_vulkan_performance(env, run)."""
    runner(
        [
            "cargo",
            "build",
            "-p",
            PACKAGE,
            "--release",
            "--example",
            "renderer_performance",
            "--locked",
        ],
        environment,
    )
