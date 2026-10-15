#!/usr/bin/env python3
"""Verify mapped toolkit suites against unchanged OCCT GoogleTest sources.

Usage: python3 tools/verify_toolkit.py TKMath --jobs 4   (or TKG2d, TKG3d, TKGeomBase, TKBRep, TKGeomAlgo, TKTopAlgo,
       TKPrim, TKHelix, TKShHealing, TKBO, TKBool, TKFillet, TKOffset, TKCDF, TKLCAF, TKCAF, TKDE, TKXSBase,
       TKService, TKExpress, TKV3d)
The TKernel verifier additionally checks its approved native skip and differential corpora.
Each toolkit also runs its numeric differential verifier (NUMERIC_VERIFIERS).
"""
from __future__ import annotations
import argparse
from test_target_selection import cargo_case_command, original_case_names
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import xml.etree.ElementTree as ET
from porting import ROOT, snapshot
from tkernel_mapping import mapped_runtime_sets, listed_runtime_cases
from tkernel_native_baseline import compile_sources
from verify_tkernel import run, unique
from native_build import (CAF_CLOSURE_TARGETS, CAF_LINKED_TOOLKITS, CAF_TOOLKITS, CLOSURE_TARGETS,
                          EXPRESS_CLOSURE_TARGETS, EXPRESS_LINKED_TOOLKITS, EXPRESS_TOOLKITS,
                          DE_CLOSURE_TARGETS, DE_LINKED_TOOLKITS, DE_TOOLKITS, EXCHANGE_CLOSURE_TARGETS,
                          EXCHANGE_LINKED_TOOLKITS, EXCHANGE_TOOLKITS, LINKED_TOOLKITS,
                          OCAF_CLOSURE_TARGETS, OCAF_LINKED_TOOLKITS, OCAF_TOOLKITS,
                          VISUALIZATION_CLOSURE_TARGETS, VISUALIZATION_LINKED_TOOLKITS,
                          VISUALIZATION_TOOLKITS, V3D_CLOSURE_TARGETS, V3D_LINKED_TOOLKITS,
                          V3D_TOOLKITS, closure_flags, locked_cmake)
from toolkit_source_disabled import (SourceDisabledCase, source_disabled_rows,
                                     active_native_cases, forced_disabled_observations)
from legacy_pending_dependency import (PendingDependencyCase, implemented_classes, pending_dependency_rows,
                                       require_blockers_in_source, runnable_rows, source_with_fixtures)
from legacy_not_applicable import not_applicable_rows
from toolkit_native_skips import approved_native_skips, native_skip_observations

# The whole numeric verifier of one toolkit (both probes over its full corpus) may run this
# long. It is a hang guard, not a budget: a corpus must never be trimmed to fit it. The
# debug-build Rust probe dominates, and TKGeomBase passed 900 s once ProjLib landed.
NUMERIC_TIMEOUT_SECONDS = 3600

# The numeric differential verifier of each toolkit: script, evidence directory.
NUMERIC_VERIFIERS = {
    'TKMesh': ('tools/verify_tkmesh_numeric.py', 'target/oracle-tkmesh/numeric'),
    'TKBO': ('tools/verify_tkbo_foundations.py', 'target/oracle-tkbo-foundations/numeric'),
    'TKMath': ('tools/verify_tkmath_numeric.py', 'target/oracle-tkmath/numeric'),
    'TKG2d': ('tools/verify_tkg2d_numeric.py', 'target/oracle-tkg2d/numeric'),
    'TKG3d': ('tools/verify_tkg3d_numeric.py', 'target/oracle-tkg3d/numeric'),
    'TKGeomBase': ('tools/verify_tkgeombase_numeric.py', 'target/oracle-tkgeombase/numeric'),
    'TKBRep': ('tools/verify_tkbrep_numeric.py', 'target/oracle-tkbrep/numeric'),
    'TKGeomAlgo': ('tools/verify_tkgeomalgo_numeric.py', 'target/oracle-tkgeomalgo/numeric'),
    'TKTopAlgo': ('tools/verify_tktopalgo_numeric.py', 'target/oracle-tktopalgo/numeric'),
    'TKHelix': ('tools/verify_tkhelix_numeric.py', 'target/oracle-tkhelix/numeric'),
    'TKShHealing': ('tools/verify_tkshhealing_numeric.py', 'target/oracle-tkshhealing/numeric'),
    'TKPrim': ('tools/verify_tkprim_numeric.py', 'target/oracle-tkprim/numeric'),
    'TKHLR': ('tools/verify_tkhlr_numeric.py', 'target/oracle-tkhlr/numeric'),
    'TKFillet': ('tools/verify_tkfillet_numeric.py', 'target/oracle-tkfillet/numeric'),
    'TKOffset': ('tools/verify_tkoffset_numeric.py', 'target/oracle-tkoffset/numeric'),
    'TKService': ('tools/verify_tkservice_numeric.py', 'target/oracle-tkservice/numeric'),
    'TKExpress': ('tools/verify_tkexpress_numeric.py', 'target/oracle-tkexpress/numeric'),
    'TKBool': ('tools/verify_tkbool_numeric.py', 'target/oracle-tkbool/numeric'),
    'TKV3d': ('tools/verify_tkv3d_numeric.py', 'target/oracle-tkv3d/numeric'),
}


# The profiles without numeric probes (the OCAF toolkits and TKXSBase hold no numeric
# algorithm), and the native closure each of them links: its own toolkit and the ones below it.
PROFILES_WITHOUT_NUMERIC: dict[str, tuple[str, ...]] = {
    'TKCDF': ('TKCDF', 'TKernel'),
    'TKLCAF': OCAF_LINKED_TOOLKITS,
    'TKCAF': CAF_LINKED_TOOLKITS,
    # TKDE holds no numeric algorithm; its suite links every DE format toolkit.
    'TKDE': DE_LINKED_TOOLKITS,
    'TKXSBase': EXCHANGE_LINKED_TOOLKITS,
}

# The native closure tree each profile with its own closure builds when no prebuilt one is
# given: toolkits, build targets and tree. TKCAF adds the DRAW toolkit TKDCAF to the OCAF set.
PROFILE_CLOSURES: dict[str, tuple[tuple[str, ...], list[str], str]] = {
    'TKCDF': (OCAF_TOOLKITS, OCAF_CLOSURE_TARGETS, 'target/occt-ocaf-full'),
    'TKLCAF': (OCAF_TOOLKITS, OCAF_CLOSURE_TARGETS, 'target/occt-ocaf-full'),
    'TKCAF': (CAF_TOOLKITS, CAF_CLOSURE_TARGETS, 'target/occt-caf-full'),
    'TKDE': (DE_TOOLKITS, DE_CLOSURE_TARGETS, 'target/occt-de-full'),
    'TKXSBase': (EXCHANGE_TOOLKITS, EXCHANGE_CLOSURE_TARGETS, 'target/occt-exchange-full'),
    # TKService needs FreeType, fontconfig and Xlib, which the other closures must not require.
    'TKService': (VISUALIZATION_TOOLKITS, VISUALIZATION_CLOSURE_TARGETS, 'target/occt-visualization-full'),
    # TKExpress sits on TKernel only; its closure builds no geometry toolkit.
    'TKExpress': (EXPRESS_TOOLKITS, EXPRESS_CLOSURE_TARGETS, 'target/occt-express-full'),
    # TKV3d adds the modeling toolkits of its EXTERNLIB to the visualization base.
    'TKV3d': (V3D_TOOLKITS, V3D_CLOSURE_TARGETS, 'target/occt-v3d-full'),
}


# The numeric profiles that link their own closure instead of the geometry one.
OWN_CLOSURE_PROFILES: dict[str, tuple[str, ...]] = {
    'TKService': VISUALIZATION_LINKED_TOOLKITS,
    'TKExpress': EXPRESS_LINKED_TOOLKITS,
    'TKV3d': V3D_LINKED_TOOLKITS,
}


def linked_toolkits(toolkit: str) -> tuple[str, ...]:
    """The native libraries a profile links, dependants first; linked_toolkits('TKMath') == LINKED_TOOLKITS."""
    return PROFILES_WITHOUT_NUMERIC.get(toolkit, OWN_CLOSURE_PROFILES.get(toolkit, LINKED_TOOLKITS))


def selected_rows(toolkit: str, inventory: dict) -> list[dict]:
    """Reject stale, partial, or unsupported mappings before building; selected_rows('TKMath', snapshot)."""
    ledger = json.loads((ROOT/'porting/parity.json').read_text())
    rows = [row for row in ledger['tests'] if row['toolkit'] == toolkit]
    # A toolkit starts with its numeric oracle before its first suite is mapped; main()
    # then runs the numeric verifier alone and reports 0 mapped declarations.
    if not rows: return []
    candidates = {row['name'] for entry in inventory['toolkits'] if entry['name'] == toolkit
                  for row in entry['gtest_declaration_candidates']}
    if len({row['upstream_id'] for row in rows}) != len(rows): raise ValueError('Duplicate declaration mappings')
    source_disabled_rows(runnable_rows(rows))
    # A pending row lapses (and the gate fails) once a class it waits for is ported.
    pending_dependency_rows(rows, implemented_classes(ledger['api']))
    # A C++-only property (move/copy traits) runs natively only and must state its reason.
    not_applicable_rows(rows)
    if {row['upstream_id'] for row in rows} - candidates: raise ValueError('Mappings outside selected toolkit inventory')
    return rows


def declared_names(text: str) -> set[str]:
    """The `Suite.Case` of every TEST, TEST_F, TEST_P and typed declaration, whatever the spacing (a
    declaration may wrap after the comma); declared_names('TEST(A, b)') == {'A.b'}."""
    pattern = r'\b(?:TEST(?:_[FP])?|TYPED_TEST(?:_P)?)\s*\(\s*(\w+)\s*,\s*(\w+)\s*\)'
    return {f'{suite}.{case}' for suite, case in re.findall(pattern, text)}


def verify_sources(source: Path, rows: list[dict]) -> list[str]:
    """Require every declaration in each selected source; verify_sources(source, rows)."""
    paths = sorted({row['source'] for row in rows})
    for path in paths:
        content = (source/path).read_bytes()
        names = declared_names(content.decode())
        selected = [row for row in rows if row['source'] == path]
        if names != {row['upstream_id'] for row in selected}: raise ValueError(f'Incomplete source-suite mapping: {path}')
        if any(row['source_sha256'] != hashlib.sha256(content).hexdigest() for row in selected):
            raise ValueError(f'Stale source hash: {path}')
    return paths


def prepare_build(source: Path, output: Path, inventory: dict, jobs: int, environment: dict, reuse: Path | None = None, toolkit: str = 'TKMath') -> Path:
    """Build the verified native closure of a profile; prepare_build(source, out, inventory, 4, env).

    A reused build is never reconfigured or rebuilt, so parallel worktrees can share
    one native closure without racing on it; it must already contain the libraries.
    The OCAF and exchange profiles build their own closure trees, so the geometry closure keeps
    its flags.
    """
    if reuse is not None:
        if not (reuse/'CMakeCache.txt').is_file(): raise ValueError(f'No configured native build at {reuse}; expected a CMake build directory')
        return reuse
    if toolkit in PROFILE_CLOSURES:
        toolkits, targets, tree = PROFILE_CLOSURES[toolkit]
        flags = closure_flags(inventory['modules'], toolkits)
        return locked_cmake(source,ROOT/tree,flags,targets,jobs,output/'configure.log',output/'build.log',run,environment)
    build = ROOT/'target/occt-tkernel-full'
    return locked_cmake(source,build,closure_flags(inventory['modules']),CLOSURE_TARGETS,jobs,output/'configure.log',output/'build.log',run,environment)


def approved_native_failures(rows: list[dict]) -> dict[str, str]:
    """Upstream ids whose unchanged test fails in native OCCT itself, with the approved
    reason; approved_native_failures(rows) == {'Suite.Case': 'reason'}."""
    approved = {row['upstream_id']: row['native_expected_failure'] for row in rows if 'native_expected_failure' in row}
    if any(not reason.strip() for reason in approved.values()): raise ValueError('An approved native failure needs a reason')
    return approved


def check_native_cases(cases: list, approved: dict[str, str]) -> list[str]:
    """Reject every skip or unexecuted case, and every native failure outside `approved`.

    An approved case must still fail natively, so the approval lapses (and the gate fails)
    once OCCT or the toolchain changes; check_native_cases(cases, {'Suite.Case': 'why'}).
    """
    if any(case.find('skipped') is not None or case.attrib.get('status') != 'run' for case in cases):
        raise ValueError('Skipped or unexecuted native test')
    failed = {case.attrib['classname']+'.'+case.attrib['name'] for case in cases if case.find('failure') is not None}
    if failed - set(approved): raise ValueError(f'Unapproved native failures: {sorted(failed - set(approved))}')
    if set(approved) - failed: raise ValueError(f'Approved native failures now pass; remove the approval: {sorted(set(approved) - failed)}')
    return sorted(failed)


def run_native_suites(command: list, log: Path, environment: dict, has_approved_failures: bool) -> None:
    """Run the native GoogleTest binary. GoogleTest exits with 1 when any test fails, so with
    approved native failures the XML results decide (check_native_cases); a crash still fails,
    because its XML is missing or incomplete; run_native_suites(cmd, log, env, False)."""
    try:
        run(command, log, env=environment)
    except RuntimeError:
        if not has_approved_failures: raise


def native_tests(source: Path, output: Path, build: Path, paths: list[str], jobs: int, expected: set, environment: dict, approved: dict[str, str], disabled: dict[str, SourceDisabledCase] | None = None, linked: tuple[str, ...] = LINKED_TOOLKITS, skips: dict[str, str] | None = None) -> list[Path]:
    """Run complete native suites; only the approved native failures may fail and only the
    approved native skips may skip (tools/toolkit_native_skips.py); native_tests(...)."""
    compiler = re.search(r'^CMAKE_CXX_COMPILER:FILEPATH=(.*)$',(build/'CMakeCache.txt').read_text(),re.M)[1]
    common = [compiler,'-std=c++17','-O2','-pthread','-I'+str(build/'include/opencascade'),
              '-I'+str(build/'_deps/googletest-src/googletest/include')]
    objects = compile_sources(source,paths,common,output,jobs)
    libraries = [unique(build,f'lib{name}.so') for name in [*linked,'gtest_main','gtest']]
    executable = output/'legacy-tests'
    run([*common,*objects,*libraries,'-Wl,-rpath,'+str(libraries[0].parent),'-o',executable],output/'link.log',env=environment)
    listing = run([executable,'--gtest_list_tests'],output/'native-list.txt',env=environment)
    if listed_runtime_cases(listing) != expected: raise ValueError('Native runtime enumeration differs from mappings')
    (output/"native.xml").unlink(missing_ok=True)
    run_native_suites([executable,f'--gtest_output=xml:{output/"native.xml"}'],output/'native-run.log',environment,bool(approved))
    cases = ET.parse(output/'native.xml').findall('.//testcase')
    if {case.attrib['classname']+'.'+case.attrib['name'] for case in cases} != expected: raise ValueError('Incomplete native results')
    active, skipped = native_skip_observations(active_native_cases(cases, disabled or {}), skips or {})
    check_native_cases(active, approved)
    (output/'native-skips.json').write_text(json.dumps(skipped, indent=2)+'\n')
    verify_forced_disabled(executable, output, environment, disabled or {})
    return libraries


def verify_forced_disabled(executable: Path, output: Path, environment: dict[str, str], disabled: dict[str, SourceDisabledCase]) -> None:
    """Run disabled originals separately and retain their failure evidence; verify_forced_disabled(...)."""
    evidence = output/'source-disabled.json'
    evidence.unlink(missing_ok=True)
    if not disabled:
        return
    xml_path = output/'native-forced-disabled.xml'
    xml_path.unlink(missing_ok=True)
    command = [executable, '--gtest_also_run_disabled_tests', '--gtest_filter='+':'.join(sorted(disabled)),
               f'--gtest_output=xml:{xml_path}']
    run_native_suites(command, output/'native-forced-disabled.log', environment, True)
    cases = ET.parse(xml_path).findall('.//testcase')
    observations = forced_disabled_observations(cases, disabled)
    evidence.write_text(json.dumps(observations, indent=2)+'\n')


def rust_tests(output: Path, rows: list[dict], expected: set, environment: dict) -> None:
    """Require exactly the mapped Rust tests to pass; rust_tests(out, rows, expected, env)."""
    targets = sorted({(row['rust_package'],row['rust_target']) for row in rows})
    for package,target in targets:
        result = run(cargo_case_command(ROOT, package, target, ['--test-threads=1']),output/f'rust-{target}.log',env=environment)
        passed = original_case_names(set(re.findall(r'^test (\S+) \.\.\. ok$',result,re.M)), target)
        require_exact_rust_cases(package, target, passed, {name for owner,mapped,name in expected if (owner,mapped) == (package,target)})


def require_exact_rust_cases(package: str, target: str, passed: set[str], mapped: set[str]) -> None:
    """A mapped target must pass exactly its mapped tests; port-only tests live in other targets
    (tools/legacy_test_targets.py finds mixed targets); require_exact_rust_cases('p', 't', {'a'}, {'a'})."""
    if passed != mapped:
        raise ValueError(f'Rust cases missing, skipped, or unmapped: {package}/{target}: '
                         f'not passed={sorted(mapped - passed)}, unmapped={sorted(passed - mapped)}')


def main() -> None:
    """Verify the selected profile and persist evidence only after success."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('toolkit', choices=sorted([*NUMERIC_VERIFIERS, *PROFILES_WITHOUT_NUMERIC]))
    parser.add_argument('--source',type=Path,default=ROOT/'../OCCT')
    parser.add_argument('--jobs',type=int,default=2)
    parser.add_argument('--native-build',type=Path,help='Reuse this prebuilt native closure without rebuilding it')
    parser.add_argument('--only',action='append',default=[],help='Development aid: verify only sources containing this text; never a full gate')
    args = parser.parse_args()
    if os.uname().sysname != 'Linux': raise ValueError('This native oracle profile requires Linux')
    source = args.source.resolve()
    # A partial run writes elsewhere so it never deletes or overwrites full-gate evidence.
    output = ROOT/'target'/('oracle-'+args.toolkit.lower()+('-partial' if args.only else '')); output.mkdir(parents=True,exist_ok=True)
    (output/'report.json').unlink(missing_ok=True)
    inventory = snapshot(source); rows = selected_rows(args.toolkit,inventory)
    if args.only:
        rows = [row for row in rows if any(text in row['source'] for text in args.only)]
        if not rows: raise ValueError(f'--only {args.only} selects no mapped sources; expected a substring of a mapped source path')
    environment = dict(os.environ,LC_ALL='C',LANG='C')
    environment['PATH'] = str(Path.home()/'.cargo/bin')+os.pathsep+environment['PATH']
    build = prepare_build(source,output,inventory,args.jobs,environment,args.native_build,args.toolkit)
    native,rust,approved,libraries = mapped_suites(source,output,build,rows,args.jobs,environment,linked_toolkits(args.toolkit))
    numeric = None
    if args.only:
        disabled = source_disabled_rows(runnable_rows(rows))
        print(f'PARTIAL PASS (--only {args.only}): {len(rows)} declarations; '
              f'{len(native)-len(approved)-len(disabled)} native passed, '
              f'{len(disabled)} source-disabled expected failures, {len(rust)} Rust passed; not a toolkit gate')
        return
    if args.toolkit in NUMERIC_VERIFIERS:
        script, evidence = NUMERIC_VERIFIERS[args.toolkit]
        run([sys.executable,ROOT/script,'--source',source,'--native-build',build],output/'numeric-verification.log',env=environment,timeout=NUMERIC_TIMEOUT_SECONDS)
        numeric = json.loads((ROOT/evidence/'report.json').read_text())
    record_report(output,args.toolkit,inventory,rows,native,rust,libraries,numeric,approved)


def mapped_suites(source: Path, output: Path, build: Path, rows: list[dict], jobs: int, environment: dict, linked: tuple[str, ...] = LINKED_TOOLKITS) -> tuple[set, set, dict[str, str], list[Path]]:
    """Run the mapped suites natively and in Rust; return the native and Rust runtime
    cases, the approved native failures and the linked libraries. Without mapped rows
    nothing runs and the libraries are only hashed for the report;
    mapped_suites(source, out, build, rows, 4, env)."""
    if not rows:
        return set(), set(), {}, [unique(build,f'lib{name}.so') for name in linked]
    paths = verify_sources(source,rows); runnable = runnable_rows(rows)
    native,rust = mapped_runtime_sets(runnable)
    native |= set(pending_native_cases(source, rows))
    native |= set(not_applicable_rows(rows))
    approved = approved_native_failures(runnable)
    disabled = source_disabled_rows(runnable)
    skips = approved_native_skips(runnable)
    libraries = native_tests(source,output,build,paths,jobs,native,environment,approved,disabled,linked,skips)
    rust_tests(output,runnable,rust,environment)
    return native,rust,approved,libraries


def pending_native_cases(source: Path, rows: list[dict]) -> dict[str, PendingDependencyCase]:
    """Pending-dependency declarations, whose unchanged native test must still run and pass;
    each blocker must be named by its source or a local fixture header it includes;
    pending_native_cases(occt, rows)."""
    pending = pending_dependency_rows(rows, set())
    for case in pending.values():
        require_blockers_in_source(case, source_with_fixtures(source/case.source))
    return pending


def record_report(output: Path, toolkit: str, inventory: dict, rows: list[dict], native: set, rust: set, libraries: list[Path], numeric: dict | None, approved: dict[str, str]) -> None:
    """Record hashes and scope without inferring whole-toolkit acceptance; record_report(...)."""
    inputs = [p for p in (ROOT/'crates').rglob('*') if p.is_file()]
    inputs += list((ROOT/'tools/oracle').glob('*.cxx'))
    inputs += sorted((ROOT/'tools').rglob('*.py'))
    inputs += [ROOT/NUMERIC_VERIFIERS[toolkit][0]] if toolkit in NUMERIC_VERIFIERS else []
    inputs += [ROOT/'Cargo.toml',ROOT/'Cargo.lock',ROOT/'rust-toolchain.toml',ROOT/'porting/parity.json',Path(__file__),ROOT/'tools/tkernel_mapping.py',ROOT/'tools/tkernel_native_baseline.py',ROOT/'tools/verify_tkernel.py',ROOT/'tools/native_build.py']
    disabled = source_disabled_rows(runnable_rows(rows))
    pending = pending_dependency_rows(rows, set())
    inapplicable = not_applicable_rows(rows)
    observations = json.loads((output/'source-disabled.json').read_text()) if disabled else []
    skips = approved_native_skips(runnable_rows(rows))
    skipped = json.loads((output/'native-skips.json').read_text()) if skips else []
    report = {'revision':inventory['revision'],'profile':'linux-gcc-c-locale','toolkit':toolkit,
              'numeric_report':numeric,'declarations_mapped':len(rows)-len(pending)-len(inapplicable),
              'rust_pending_dependency':{name:list(case.blocked_by) for name,case in sorted(pending.items())},
              'rust_not_applicable':{name:case.reason for name,case in sorted(inapplicable.items())},'legacy_runtime_cases_passed':len(native)-len(approved)-len(disabled)-len(skips),
              'rust_runtime_cases_passed':len(rust),'toolkit_complete':False,
              'native_expected_failures':approved, 'source_disabled_cases':observations,
              'native_skipped_cases':skipped,
              'scope':'Mapped GoogleTest suites only; full toolkit API and test inventories pending',
              'source_hashes':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs},
              'oracle_libraries_sha256':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in libraries}}
    (output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    failures = f' ({len(approved)} approved native failures)' if approved else ''
    print(f'PASS: {toolkit}: {len(rows)-len(pending)} mapped declarations, {len(native)-len(approved)-len(disabled)-len(skips)} native and {len(rust)} Rust runtime cases{failures}; '
          f'{len(disabled)} source-disabled expected failures; {len(skips)} approved native skips; {len(pending)} pending dependency (native passed, Rust pending)')


if __name__ == '__main__':
    main()
