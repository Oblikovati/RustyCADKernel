#!/usr/bin/env python3
"""Capture and cross-check original fixed-seed mutation inputs; run with --help."""
from __future__ import annotations
import argparse
from pathlib import Path
import re
import subprocess
from native_build import LINKED_TOOLKITS
from porting import ROOT, snapshot
from verify_tkernel import unique


def compiler_arguments(build: Path, source: Path) -> list[str]:
    """Use the oracle compiler and unchanged test source; compiler_arguments(build, source)."""
    cache = (build / 'CMakeCache.txt').read_text()
    compiler = re.search(r'^CMAKE_CXX_COMPILER:FILEPATH=(.*)$', cache, re.M)[1]
    return [compiler, '-std=c++17', '-O2', '-pthread',
            '-I' + str(build / 'include/opencascade'),
            '-I' + str(build / '_deps/googletest-src/googletest/include'),
            '-I' + str(source / 'src/ModelingData/TKBRep/GTests')]


def capture_inputs(source: Path, build: Path, output: Path) -> None:
    """Compile the trace verifier without rebuilding OCCT; capture_inputs(source, build, output)."""
    snapshot(source)
    libraries = [unique(build, f'lib{name}.so') for name in [*LINKED_TOOLKITS, 'gtest']]
    scratch = ROOT / 'target/tkbrep-fuzz-inputs'
    scratch.mkdir(parents=True, exist_ok=True)
    executable = scratch / 'capture'
    command = [*compiler_arguments(build, source), ROOT / 'tools/oracle/tkbrep_fuzz_inputs.cxx',
               *libraries, '-Wl,-rpath,' + str(libraries[0].parent), '-o', executable]
    subprocess.run(command, check=True)
    output.mkdir(parents=True, exist_ok=True)
    subprocess.run([executable, output / 'box.txt', output / 'cylinder.txt'], check=True)


def main() -> None:
    """Parse explicit source/build paths; invoke this script with --source OCCT --native-build BUILD."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--native-build', type=Path, required=True)
    parser.add_argument('--output', type=Path, default=ROOT / 'crates/tkbrep/tests/graph_fuzz/inputs')
    args = parser.parse_args()
    capture_inputs(args.source.resolve(), args.native_build.resolve(), args.output.resolve())


if __name__ == '__main__':
    main()
