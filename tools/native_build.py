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

# The clean geometry closure exceeded 900s on the two-core hosted runner in
# Actions run 38077155403. This build guard is separate from test process limits.
NATIVE_BUILD_TIMEOUT_SECONDS = 3600

# Toolkits the shared oracle closure builds beyond FoundationClasses (with their
# dependencies). TKPrim is here because unchanged TKBRep suites (TopExp_Test) build their
# shapes with BRepPrimAPI_MakeBox; TKHelix and TKShHealing for their own suites and
# numeric oracles.
ADDITIONAL_TOOLKITS: tuple[str, ...] = ('TKBO', 'TKTopAlgo', 'TKPrim', 'TKMesh',
                                        'TKHelix',
                                        'TKShHealing',
                                        'TKHLR',
                                        'TKOffset',
                                        )
# The closure's shared libraries in link order, dependants first. TKOffset is linked only for
# the TKGeomAlgo GTest GeomFill_BSplineCurves_Test (OCC28131 offsets the filled face); it is
# also an additional toolkit, so every closure build (verify_tkernel too) produces the library.
# TKFillet is linked for the TKBO GTest BRepAlgoAPI_Fuse_Test (BOPTest_Utilities::CreateBlend
# builds a BRepFilletAPI_MakeFillet); TKOffset depends on it, so the closure builds it too.
# TKBool (BRepFill, BRepAlgo, TopOpeBRep...) sits between TKOffset/TKFillet and TKBO;
# TKOffset depends on it, so the closure builds it too.
LINKED_TOOLKITS: tuple[str, ...] = ('TKOffset', 'TKFillet', 'TKBool', 'TKBO', 'TKMesh', 'TKPrim',
                                    'TKHelix',
                                    'TKShHealing',
                                        'TKHLR',
                                    'TKTopAlgo', 'TKGeomAlgo', 'TKBRep', 'TKGeomBase',
                                    'TKG3d', 'TKG2d', 'TKMath', 'TKernel')
# The targets that build the whole closure plus GoogleTest: every additional toolkit,
# since no single one of them depends on all the others (TKPrim, TKHelix and TKShHealing
# are siblings over TKTopAlgo).
CLOSURE_TARGETS: list[str] = [*ADDITIONAL_TOOLKITS, 'gtest_main']


# The OCAF oracle closure (TKCDF and TKLCAF suites) lives in its own build tree, so the
# geometry closure above keeps its flags. TKBin is here because the unchanged TKLCAF suites
# call BinDrivers::DefineFormat; TKCAF for TNaming and TDataXtd; TKPrim for their boxes.
OCAF_TOOLKITS: tuple[str, ...] = ('TKBin', 'TKCAF', 'TKPrim')
# The OCAF closure's shared libraries in link order, dependants first (TKCAF sits on TKBO).
OCAF_LINKED_TOOLKITS: tuple[str, ...] = ('TKBin', 'TKBinL', 'TKCAF', 'TKBO', 'TKPrim', 'TKTopAlgo',
                                         'TKGeomAlgo', 'TKBRep', 'TKGeomBase', 'TKG3d', 'TKG2d',
                                         'TKLCAF', 'TKCDF', 'TKMath', 'TKernel')
OCAF_CLOSURE_TARGETS: list[str] = [*OCAF_TOOLKITS, 'gtest_main']

# The TKCAF closure: the unchanged TNaming_Tool_Test drives DNaming functions (TKDCAF, a DRAW
# toolkit over Tcl/Tk), whose drivers call Boolean operations (TKBool) and fillets (TKFillet).
# It lives in its own tree too, so the OCAF closure keeps its flags.
CAF_TOOLKITS: tuple[str, ...] = (*OCAF_TOOLKITS, 'TKDCAF', 'TKFillet', 'TKBool')
# The TKCAF closure's shared libraries in link order, dependants first (TKDCAF sits on TKCAF).
CAF_LINKED_TOOLKITS: tuple[str, ...] = ('TKDCAF', 'TKFillet', 'TKBool', *OCAF_LINKED_TOOLKITS)
CAF_CLOSURE_TARGETS: list[str] = [*CAF_TOOLKITS, 'gtest_main']

# The TKTObj closure: the unchanged TObj_Model_Test registers the BinTObj format
# (BinTObjDrivers::DefineFormat) for its two stream cases, so TKBinTObj is built with it,
# although the Rust port leaves that native format out of scope. It lives in its own tree,
# so the other closures keep their flags.
TOBJ_TOOLKITS: tuple[str, ...] = ('TKBinTObj',)
# The TKTObj closure's shared libraries in link order, dependants first (the EXTERNLIB of
# TKBinTObj, TKBinL and TKTObj).
TOBJ_LINKED_TOOLKITS: tuple[str, ...] = ('TKBinTObj', 'TKBinL', 'TKTObj', 'TKLCAF', 'TKCDF',
                                         'TKMath', 'TKernel')
TOBJ_CLOSURE_TARGETS: list[str] = [*TOBJ_TOOLKITS, 'gtest_main']

# The TKDE closure: the unchanged DE_Wrapper_Configuration_Test binds the configuration node of
# every exchange format, so it links the DE format toolkits (TKDECascade holds DEBREP and DEXCAF).
# It lives in its own tree too, so the other closures keep their flags.
DE_TOOLKITS: tuple[str, ...] = ('TKDECascade', 'TKDEGLTF', 'TKDEIGES', 'TKDEOBJ', 'TKDEPLY',
                                'TKDESTEP', 'TKDESTL', 'TKDEVRML')
# The TKDE closure's shared libraries in link order, dependants first (the formats sit on TKDE).
DE_LINKED_TOOLKITS: tuple[str, ...] = (*DE_TOOLKITS, 'TKDE', 'TKBRep', 'TKMath', 'TKernel')
DE_CLOSURE_TARGETS: list[str] = [*DE_TOOLKITS, 'gtest_main']

# The exchange oracle closure (TKXSBase, the foundation of the STEP and IGES toolkits) lives
# in its own build tree too, so the geometry closure keeps its flags. TKDE is built with it:
# XSAlgo_ShapeProcessor.hxx includes DE_ShapeFixParameters.hxx although TKXSBase's EXTERNLIB
# omits TKDE, and a partial tree installs only the headers of the toolkits it builds.
EXCHANGE_TOOLKITS: tuple[str, ...] = ('TKDE', 'TKXSBase')
# The exchange closure's shared libraries in link order, dependants first (the EXTERNLIB of
# TKXSBase, TKDE and TKShHealing's own TKGeomAlgo).
EXCHANGE_LINKED_TOOLKITS: tuple[str, ...] = ('TKXSBase', 'TKDE', 'TKShHealing', 'TKTopAlgo',
                                             'TKGeomAlgo', 'TKBRep', 'TKGeomBase', 'TKG3d', 'TKG2d',
                                             'TKMath', 'TKernel')
EXCHANGE_CLOSURE_TARGETS: list[str] = [*EXCHANGE_TOOLKITS, 'gtest_main']


# The visualization oracle closure (TKService suites and numeric oracle) lives in its own
# build tree too: TKService needs FreeType, fontconfig and Xlib, which the geometry and
# OCAF closures must not require. The shared all-modules build can be reused instead.
VISUALIZATION_TOOLKITS: tuple[str, ...] = ('TKService',)
VISUALIZATION_LINKED_TOOLKITS: tuple[str, ...] = ('TKService', 'TKMath', 'TKernel')
VISUALIZATION_CLOSURE_TARGETS: list[str] = [*VISUALIZATION_TOOLKITS, 'gtest_main']

# The expression oracle closure (TKExpress suites and numeric oracle): TKExpress sits on
# TKernel only (its EXTERNLIB), so its tree holds no geometry toolkit. The shared
# all-modules build can be reused instead.
EXPRESS_TOOLKITS: tuple[str, ...] = ('TKExpress',)
EXPRESS_LINKED_TOOLKITS: tuple[str, ...] = ('TKExpress', 'TKernel')
EXPRESS_CLOSURE_TARGETS: list[str] = [*EXPRESS_TOOLKITS, 'gtest_main']
# The TKV3d oracle closure (TKV3d suites and numeric oracle) lives in its own build tree too:
# TKV3d sits on TKService (FreeType, fontconfig, Xlib) and on the modeling toolkits of its
# EXTERNLIB. TKPrim is here because the unchanged AIS_ColoredShape_Test suite builds its
# shapes with BRepPrimAPI_MakeBox (outside TKV3d's EXTERNLIB). The shared all-modules build
# can be reused instead.
V3D_TOOLKITS: tuple[str, ...] = ('TKV3d', 'TKPrim')
# The TKV3d closure's shared libraries in link order, dependants first (its EXTERNLIB, with
# TKPrim for the GTests).
V3D_LINKED_TOOLKITS: tuple[str, ...] = ('TKV3d', 'TKService', 'TKHLR', 'TKMesh', 'TKShHealing',
                                        'TKPrim', 'TKTopAlgo', 'TKGeomAlgo', 'TKBRep',
                                        'TKGeomBase', 'TKG3d', 'TKG2d', 'TKMath', 'TKernel')
V3D_CLOSURE_TARGETS: list[str] = [*V3D_TOOLKITS, 'gtest_main']
# The TKMeshVS oracle closure (numeric oracle; the toolkit has no GTest suite at the pinned
# revision) adds TKMeshVS to the TKV3d tree. The shared all-modules build can be reused instead.
MESHVS_TOOLKITS: tuple[str, ...] = ('TKMeshVS',)
# The TKMeshVS closure's shared libraries in link order, dependants first (its EXTERNLIB on
# top of the TKV3d closure).
MESHVS_LINKED_TOOLKITS: tuple[str, ...] = ('TKMeshVS', *V3D_LINKED_TOOLKITS)
MESHVS_CLOSURE_TARGETS: list[str] = [*MESHVS_TOOLKITS, 'gtest_main']


def closure_flags(modules: list[str], toolkits: tuple[str, ...] = ADDITIONAL_TOOLKITS) -> list[str]:
    """CMake flags of a shared oracle closure (the geometry one by default); one spelling for
    every tool, so the tools never reconfigure the tree back and forth;
    closure_flags(snapshot(source)['modules'])."""
    flags = ['-DCMAKE_BUILD_TYPE=RelWithDebInfo', '-DCMAKE_EXPORT_COMPILE_COMMANDS=ON',
             '-DBUILD_GTEST=ON', '-DCMAKE_DISABLE_FIND_PACKAGE_GTest=ON',
             '-DGTEST_USE_FETCHCONTENT=ON',
             '-DBUILD_ADDITIONAL_TOOLKITS=' + ' '.join(toolkits)]
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
            build_log, env=env, timeout=NATIVE_BUILD_TIMEOUT_SECONDS)
    return canonical
