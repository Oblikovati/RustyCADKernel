#!/usr/bin/env python3
"""Run Linux test commands with the verified validation layer, without a system install."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import platform
import subprocess

from vulkan_validation_cache import ValidationCacheIO, VerifiedValidationCache, noble_validation_pin


class NativeValidationCacheIO:
    def exists(self, path: Path) -> bool:
        """Inspect a cached artifact.
        Example: `io.exists(path)`.
        """
        return path.is_file()

    def read(self, path: Path) -> bytes:
        """Read checksum inputs.
        Example: `io.read(path)`.
        """
        return path.read_bytes()

    def download(self, url: str, path: Path) -> None:
        """Fetch only the minimal pinned package; e.g. `io.download(url, archive)`."""
        path.parent.mkdir(parents=True, exist_ok=True)
        temporary = path.with_suffix(".download")
        subprocess.run(
            ["curl", "--fail", "--location", "--retry", "2", "--max-time", "60",
             "--output", str(temporary), url], check=True,
        )
        temporary.replace(path)

    def extract(self, archive: Path, destination: Path) -> None:
        """Extract under the caller's cache; e.g. `io.extract(archive, folder)`."""
        destination.mkdir(parents=True, exist_ok=True)
        subprocess.run(["dpkg-deb", "-x", str(archive), str(destination)], check=True)


def require_noble_platform(system: str, machine: str) -> None:
    """Keep the binary package's platform explicit; e.g. `require_noble_platform('Linux', 'x86_64')`."""
    if system == "Linux" and machine == "x86_64":
        return
    raise ValueError(f"platform={(system, machine)!r}; expected Linux x86_64 for the noble package")


def command_environment(
    cache: Path, inherited: dict[str, str], cache_io: ValidationCacheIO,
    system: str, machine: str,
) -> dict[str, str]:
    """Prepare the pinned test dependency; e.g. `command_environment(cache, env, io, 'Linux', 'x86_64')`."""
    require_noble_platform(system, machine)
    paths = VerifiedValidationCache(cache, noble_validation_pin(), cache_io).prepare()
    return paths.environment(inherited)


# Cargo's test runner starts children in each package directory; keep one cache at the workspace root.
WORKSPACE_VALIDATION_CACHE = Path(__file__).resolve().parents[1] / "target" / "vulkan-validation"


def validation_arguments(argv: list[str] | None) -> argparse.Namespace:
    """Parse wrapper and child arguments; e.g. `validation_arguments(['--', 'cargo', 'test-all'])`."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache", type=Path, default=WORKSPACE_VALIDATION_CACHE)
    parser.add_argument("--github-env", type=Path)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    if not command and not args.github_env:
        parser.error("command=[]; expected an executable and optional arguments after --")
    args.command = command
    return args


def export_github_environment(path: Path, environment: dict[str, str]) -> None:
    """Select the same dependency for workflow steps; e.g. `export_github_environment(path, env)`."""
    with path.open("a") as output:
        for key in ("VK_LAYER_PATH", "LD_LIBRARY_PATH"):
            output.write(f"{key}={environment[key]}\n")


def main(argv: list[str] | None = None) -> int:
    args = validation_arguments(argv)
    environment = command_environment(
        args.cache, dict(os.environ), NativeValidationCacheIO(), platform.system(), platform.machine(),
    )
    if args.github_env:
        export_github_environment(args.github_env, environment)
        return 0
    return shell_exit_status(subprocess.run(args.command, env=environment).returncode)


def shell_exit_status(returncode: int) -> int:
    """Report a signalled child like a shell so crashes stay visible; e.g. `shell_exit_status(-11) == 139`."""
    return 128 - returncode if returncode < 0 else returncode


if __name__ == "__main__":
    raise SystemExit(main())
