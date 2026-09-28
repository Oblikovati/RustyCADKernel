"""Configure and build a shared native OCCT tree without races between worktrees.

Worktrees reach the shared build through a symlink. CMake records the path it was
configured from, so two worktrees that pass different spellings of the same tree
reconfigure it back and forth, and parallel builds then delete each other's outputs
(for example a missing libgtest.so.1.16.0). Resolving the path gives every caller one
spelling, and an exclusive lock lets only one configure-and-build run at a time.

Usage:
    build = locked_cmake(source, ROOT/'target/occt-tkernel-full', flags,
                         ['TKTopAlgo', 'gtest_main'], 16, output/'configure.log',
                         output/'build.log', run, env)
"""
from __future__ import annotations

import fcntl
from pathlib import Path
from typing import Callable

Runner = Callable[..., str]


def lock_path(build: Path) -> Path:
    """Return the lock file beside the canonical build; lock_path(Path('/b/occt'))."""
    return build.parent/f'.{build.name}.lock'


def locked_cmake(source: Path, build: Path, flags: list[str], targets: list[str], jobs: int,
                 configure_log: Path, build_log: Path, run: Runner, env: dict[str, str]) -> Path:
    """Configure and build `build` under an exclusive lock; return its resolved path."""
    canonical = build.resolve()
    canonical.mkdir(parents=True, exist_ok=True)
    with lock_path(canonical).open('w') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        run(['cmake', '-S', source, '-B', canonical, *flags], configure_log, env=env)
        run(['cmake', '--build', canonical, '--target', *targets, '--parallel', jobs],
            build_log, env=env)
    return canonical
