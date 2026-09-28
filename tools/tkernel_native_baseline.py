#!/usr/bin/env python3
"""Build every unchanged TKernel GoogleTest source and record runtime expansion.

Usage: python3 tools/tkernel_native_baseline.py --jobs 4
A skipped native regression remains a skip; this is not Rust acceptance evidence.
"""
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import os
from pathlib import Path
import subprocess
import xml.etree.ElementTree as ET

from porting import ROOT, snapshot


def execute(command: list[str], log: Path) -> None:
    """Persist diagnostics and propagate failure; execute(['cmake','--version'], log)."""
    with log.open('w') as stream:
        subprocess.run(command, cwd=ROOT, stdout=stream, stderr=stream, check=True,
                       env=dict(os.environ, LC_ALL="C", LANG="C"))


def configure(source: Path, build: Path, output: Path, modules: list[str]) -> None:
    """Configure the native dependency closure; configure(source, build, output, modules)."""
    flags = ['-DCMAKE_BUILD_TYPE=RelWithDebInfo', '-DBUILD_GTEST=ON',
             '-DCMAKE_DISABLE_FIND_PACKAGE_GTest=ON', '-DGTEST_USE_FETCHCONTENT=ON',
             '-DBUILD_ADDITIONAL_TOOLKITS=TKTopAlgo']
    flags += [f'-DBUILD_MODULE_{name}={"ON" if name == "FoundationClasses" else "OFF"}'
              for name in modules]
    execute(['cmake', '-S', str(source), '-B', str(build), *flags], output/'configure.log')


def compile_sources(source: Path, sources: list[str], common: list[str], output: Path,
                    jobs: int) -> list[Path]:
    """Compile each inventoried source unchanged; compile_sources(source, paths, flags, out, 4)."""
    def compile_one(relative: str) -> Path:
        translation = source/relative
        obj = output/(translation.stem+'.o')
        execute([*common, '-c', str(translation), '-o', str(obj)],
                output/(translation.stem+'.log'))
        return obj
    with ThreadPoolExecutor(max_workers=jobs) as pool:
        objects = list(pool.map(compile_one, sources))
    if not objects or len(objects) != len(sources):
        raise RuntimeError('Incomplete native translation-unit set')
    return objects


def record_runtime(source: Path, output: Path, revision: str, sources: list[str]) -> None:
    """Record pass/skip/failure without reinterpretation; record_runtime(source, out, sha, paths)."""
    cases = ET.parse(output/'upstream.xml').findall('.//testcase')
    if not cases:
        raise RuntimeError('Native execution produced no runtime cases')
    records = [{'name': c.attrib['classname']+'.'+c.attrib['name'],
                'type': c.attrib.get('type_param'),
                'status': 'failed' if c.find('failure') is not None else
                          'skipped' if c.find('skipped') is not None else 'passed',
                'skip_reason': c.find('skipped').attrib.get('message') if c.find('skipped') is not None else None}
               for c in cases]
    report = {'revision': revision, 'profile': 'linux-gcc-c-locale', 'cases': records,
              'source_hashes': {p: hashlib.sha256((source/p).read_bytes()).hexdigest() for p in sources},
              'rust_acceptance': False}
    (output/'runtime-inventory.json').write_text(json.dumps(report, indent=2)+'\n')
    print(f"Native runtime: {len(cases)} cases; {sum(r['status']=='passed' for r in records)} passed; "
          f"{sum(r['status']=='skipped' for r in records)} skipped", flush=True)


def build_runtime(source: Path, build: Path, output: Path, sources: list[str], jobs: int) -> None:
    """Link only TKernel tests against their native dependencies; build_runtime(source, build, out, paths, 4)."""
    execute(['cmake', '--build', str(build), '--target', 'TKTopAlgo', 'gtest_main',
             '--parallel', str(jobs)], output/'build.log')
    common = ['c++', '-std=c++17', '-O2', '-pthread', '-I'+str(build/'include/opencascade'),
              '-I'+str(build/'_deps/googletest-src/googletest/include')]
    objects = compile_sources(source, sources, common, output, jobs)
    libraries = build/'lin64/gcc/libi'
    names = ['TKTopAlgo', 'TKGeomAlgo', 'TKBRep', 'TKGeomBase', 'TKG3d', 'TKG2d', 'TKMath',
             'TKernel', 'gtest_main', 'gtest']
    executable = output/'legacy-tests'
    execute([*common, *map(str, objects), '-L'+str(libraries), *['-l'+n for n in names],
             '-Wl,-rpath,'+str(libraries), '-o', str(executable)], output/'link.log')
    execute([str(executable), '--gtest_list_tests'], output/'runtime-list.txt')
    execute([str(executable), '--gtest_output=xml:'+str(output/'upstream.xml')], output/'run.log')


def main() -> None:
    """Run the pinned Linux oracle inventory; invoked from the command line."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=ROOT/'../OCCT')
    parser.add_argument('--jobs', type=int, default=4)
    args = parser.parse_args()
    source = args.source.resolve()
    inventory = snapshot(source)
    sources = next(t['gtest_sources'] for t in inventory['toolkits'] if t['name']=='TKernel')
    output = ROOT/'target/oracle-tkernel/full'
    output.mkdir(parents=True, exist_ok=True)
    (output/'runtime-inventory.json').unlink(missing_ok=True)
    build = ROOT/'target/occt-tkernel-full'
    configure(source, build, output, inventory['modules'])
    build_runtime(source, build, output, sources, args.jobs)
    record_runtime(source, output, inventory['revision'], sources)


if __name__ == '__main__':
    main()
