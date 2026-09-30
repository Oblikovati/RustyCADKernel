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

# Toolkits the shared oracle closure builds beyond FoundationClasses (with their
# dependencies). TKPrim is here because unchanged TKBRep suites (TopExp_Test) build their
# shapes with BRepPrimAPI_MakeBox; TKHelix and TKShHealing for their own suites and
# numeric oracles.
ADDITIONAL_TOOLKITS: tuple[str, ...] = ('TKTopAlgo', 'TKPrim',
                                        'TKHelix',
                                        'TKShHealing',
                                        'TKHLR',
                                        )
# The closure's shared libraries in link order, dependants first.
LINKED_TOOLKITS: tuple[str, ...] = ('TKPrim',
                                    'TKHelix',
                                    'TKShHealing',
                                        'TKHLR',
                                    'TKTopAlgo', 'TKGeomAlgo', 'TKBRep', 'TKGeomBase',
                                    'TKG3d', 'TKG2d', 'TKMath', 'TKernel')
# The targets that build the whole closure plus GoogleTest: every additional toolkit,
# since no single one of them depends on all the others (TKPrim, TKHelix and TKShHealing
# are siblings over TKTopAlgo).
CLOSURE_TARGETS: list[str] = [*ADDITIONAL_TOOLKITS, 'gtest_main']


def closure_flags(modules: list[str]) -> list[str]:
    """CMake flags of the shared oracle closure; one spelling for every tool, so the tools
    never reconfigure the tree back and forth; closure_flags(snapshot(source)['modules'])."""
    flags = ['-DCMAKE_BUILD_TYPE=RelWithDebInfo', '-DCMAKE_EXPORT_COMPILE_COMMANDS=ON',
             '-DBUILD_GTEST=ON', '-DCMAKE_DISABLE_FIND_PACKAGE_GTest=ON',
             '-DGTEST_USE_FETCHCONTENT=ON',
             '-DBUILD_ADDITIONAL_TOOLKITS=' + ' '.join(ADDITIONAL_TOOLKITS)]
    return flags + [f'-DBUILD_MODULE_{name}={"ON" if name == "FoundationClasses" else "OFF"}'
                    for name in modules]


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
