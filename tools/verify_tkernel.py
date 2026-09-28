#!/usr/bin/env python3
"""Build unchanged mapped OCCT test sources and verify their Rust translations on Linux."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import random
import re
import subprocess
import sys
import xml.etree.ElementTree as ET

from porting import ROOT, snapshot
from tkernel_mapping import mapped_runtime_sets, listed_runtime_cases, verified_native_skips
from tkernel_native_baseline import compile_sources
import tkernel_ncollection_corpus
import tkernel_quantity_corpus


def run(args, log, *, input=None, timeout=900, env=None):
    result = subprocess.run([str(a) for a in args], cwd=ROOT, text=True, input=input,
                            capture_output=True, timeout=timeout, env=env)
    log.write_text(result.stdout + result.stderr)
    if result.returncode:
        raise RuntimeError(f"Command failed ({result.returncode}): {' '.join(map(str, args))}; see {log}")
    return result.stdout


def unique(root, name):
    paths = list(root.rglob(name))
    if len(paths) != 1:
        raise RuntimeError(f"Expected exactly one {name} under {root}, found {len(paths)}")
    return paths[0]


def corpus():
    # Deliberately avoid C++ signed-overflow domains: those have no defined oracle value.
    cases = ["period 0 0 0 0 0 0", "period 1 -1 0 0 0 0", "period 0 0 0 -1 0 0",
             "period 0 0 0 0 -1 1000", "period 0 0 0 0 0 -1",
             "period_math 0 999999 0 1", "period_math 0 0 0 0",
             "date 1 1 1979 0 0 0 0 0", "date 2 29 2000 0 0 0 0 0",
             "date 2 29 2001 0 0 0 0 0", "date 1 1 1978 0 0 0 0 0",
             "date 0 1 2020 0 0 0 0 0", "date 13 1 2020 0 0 0 0 0",
             "date 1 0 2020 0 0 0 0 0", "date 4 31 2020 0 0 0 0 0",
             "date_sub 1 1 1979 0 0 0 0 0 0 1",
             "date_add 12 31 2020 23 59 59 999 999 0 1",
             "date_diff 1 1 1979 0 0 0 0 0 1 1 1979 0 0 0 0 1",
             "guid EMPTY", "guid 00000000-0000-0000-0000-000000000000",
             "guid FFFFFFFF-FFFF-FFFF-FFFF-FFFFFFFFFFFF",
             "guid 0000000G-0000-0000-0000-000000000000"]
    # All time-component lower/upper validation limits.
    for index, values in [(3, [-1, 24]), (4, [-1, 60]), (5, [-1, 60]), (6, [-1, 1000]), (7, [-1, 1000])]:
        for value in values:
            fields = [1, 1, 2020, 0, 0, 0, 0, 0]
            fields[index] = value
            cases.append("date " + " ".join(map(str, fields)))
    rng = random.Random(1979)
    def date():
        return f"{rng.randint(1,12)} {rng.randint(1,28)} {rng.randint(1979,2040)} {rng.randrange(24)} {rng.randrange(60)} {rng.randrange(60)} {rng.randrange(1000)} {rng.randrange(1000)}"
    for _ in range(256):
        cases.append("period " + " ".join(map(str, [rng.randrange(1000), rng.randrange(100), rng.randrange(100), rng.randrange(100), rng.randrange(2000), rng.randrange(3000000)])))
        cases.append("period_math " + " ".join(str(rng.randrange(n)) for n in [1000000,3000000,1000000,3000000]))
        a, b = date(), date()
        cases.extend([f"date {a}", f"date_diff {a} {b}", f"date_add {a} {rng.randrange(1000000)} {rng.randrange(3000000)}", f"date_sub {a} {rng.randrange(1000000)} {rng.randrange(3000000)}"])
        value = f"{rng.getrandbits(128):032x}"
        cases.append(f"guid {value[:8]}-{value[8:12]}-{value[12:16]}-{value[16:20]}-{value[20:]}")
    cases += tkernel_ncollection_corpus.cases()
    cases += tkernel_quantity_corpus.cases()
    return "\n".join(cases) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=ROOT / "../OCCT")
    parser.add_argument("--jobs", type=int, default=2)
    parser.add_argument("--require-complete", action="store_true", help="Fail unless the entire TKernel legacy suite is accounted for")
    args = parser.parse_args()
    if sys.platform != "linux":
        raise RuntimeError("This oracle profile is Linux-only; cross-platform Rust tests run separately")
    output = ROOT / "target/oracle-tkernel"
    output.mkdir(parents=True, exist_ok=True)
    report_path = output / "report.json"
    report_path.unlink(missing_ok=True)  # Never retain a success report after a failed rerun.
    source = args.source.resolve()
    inventory = snapshot(source)  # Checks exact revision, cleanliness and complete toolkit accounting.
    ledger = json.loads((ROOT / "porting/parity.json").read_text())
    mappings = ledger["tests"]
    mappings = [m for m in mappings if m["toolkit"] == "TKernel"]
    if not mappings:
        raise RuntimeError("No TKernel mappings; refusing an empty success")
    all_candidates = {c["name"] for t in inventory["toolkits"] if t["name"] == "TKernel" for c in t["gtest_declaration_candidates"]}
    missing = all_candidates - {m["upstream_id"] for m in mappings}
    if args.require_complete and (missing or not ledger["runtime_test_inventory_complete"] or not ledger["api_inventory_complete"]):
        raise RuntimeError(f"TKernel NOT COMPLETE: {len(missing)} unmapped GoogleTest declarations; full runtime/DRAW and API inventories also required")
    print(f"TKernel progress: {len(mappings)}/{len(all_candidates)} discovered declarations mapped; toolkit acceptance pending", flush=True)
    sources = sorted({m["source"] for m in mappings})
    expected, rust_expected = mapped_runtime_sets(mappings)
    if len({m["upstream_id"] for m in mappings}) != len(mappings):
        raise RuntimeError("Duplicate upstream test mappings")
    for path in sources:
        data = (source / path).read_bytes()
        declarations = {f"{s}.{c}" for s,c in re.findall(r"(?:TEST(?:_F)?|TYPED_TEST)\((\w+), (\w+)\)",data.decode())}
        rows = [m for m in mappings if m["source"] == path]
        if declarations != {m["upstream_id"] for m in rows}:
            raise RuntimeError(f"Incomplete source-suite mapping: {path}")
        if any(m["source_sha256"] != hashlib.sha256(data).hexdigest() for m in rows):
            raise RuntimeError(f"Stale source mapping: {path}")
    build = ROOT / "target/occt-foundation"
    env = dict(os.environ, LC_ALL="C", LANG="C")
    env["PATH"] = str(Path(os.environ.get("CARGO_HOME", Path.home()/".cargo"))/"bin") + os.pathsep + env["PATH"]
    flags = ["-DCMAKE_BUILD_TYPE=RelWithDebInfo", "-DCMAKE_EXPORT_COMPILE_COMMANDS=ON", "-DBUILD_GTEST=ON", "-DCMAKE_DISABLE_FIND_PACKAGE_GTest=ON", "-DGTEST_USE_FETCHCONTENT=ON", "-DBUILD_ADDITIONAL_TOOLKITS="]
    flags += [f"-DBUILD_MODULE_{module}={'ON' if module == 'FoundationClasses' else 'OFF'}" for module in inventory["modules"]]
    print("Building TKernel and GoogleTest for the focused oracle", flush=True)
    run(["cmake", "-S", source, "-B", build, *flags], output/"configure.log", env=env)
    run(["cmake", "--build", build, "--target", "TKernel", "gtest_main", "--parallel", args.jobs], output/"build.log", env=env)
    if any(m["rust_target"] in {"standard_dump", "toploc_location"} for m in mappings):
        build = ROOT / "target/occt-tkernel-full"
        flags = [flag.replace("-DBUILD_ADDITIONAL_TOOLKITS=", "-DBUILD_ADDITIONAL_TOOLKITS=TKTopAlgo") if flag == "-DBUILD_ADDITIONAL_TOOLKITS=" else flag for flag in flags]
        print("Building the geometry dependency closure used by TKernel legacy tests", flush=True)
        run(["cmake", "-S", source, "-B", build, *flags], output/"configure-geometry.log", env=env)
        run(["cmake", "--build", build, "--target", "TKTopAlgo", "gtest_main", "--parallel", args.jobs], output/"build-geometry.log", env=env)
    compiler = re.search(r"^CMAKE_CXX_COMPILER:FILEPATH=(.*)$", (build/"CMakeCache.txt").read_text(), re.M)[1]
    kernel = unique(build, "libTKernel.so")
    gtest = unique(build, "libgtest.so")
    gtest_main = unique(build, "libgtest_main.so")
    include = build/"include/opencascade"
    gtest_include = build/"_deps/googletest-src/googletest/include"
    if not gtest_include.is_dir():
        raise RuntimeError("This profile requires CMake-fetched GoogleTest; external installations need their own profile")
    common = [compiler,"-std=c++17","-O2","-pthread",f"-I{include}", f"-I{gtest_include}"]
    libraries = [kernel,gtest_main,gtest]
    if build.name == "occt-tkernel-full":
        libraries = [unique(build, f"lib{name}.so") for name in ["TKTopAlgo", "TKGeomAlgo", "TKBRep", "TKGeomBase", "TKG3d", "TKG2d", "TKMath"]] + libraries
    rpaths = [f"-Wl,-rpath,{path}" for path in sorted({p.parent for p in libraries})]
    legacy = output/"legacy-tests"
    objects = compile_sources(source, sources, common, output, args.jobs)
    run([*common, *objects, *libraries, *rpaths, "-o", legacy], output/"compile-tests.log", env=env)
    listing = run([legacy,"--gtest_list_tests"],output/"upstream-tests.txt",env=env)
    discovered = listed_runtime_cases(listing)
    if discovered != expected:
        raise RuntimeError("Upstream runtime discovery does not match mapped tests")
    run([legacy,f"--gtest_output=xml:{output/'upstream.xml'}"],output/"upstream-run.log",env=env)
    cases = ET.parse(output/"upstream.xml").findall(".//testcase")
    if {c.attrib['classname']+'.'+c.attrib['name'] for c in cases} != expected:
        raise RuntimeError("Missing, failed or skipped upstream tests")
    native_skips = verified_native_skips(cases, mappings)
    native_passed = len(cases) - len(native_skips)
    print(f"OCCT: {native_passed} legacy tests passed; {len(native_skips)} explicit upstream skips; running mapped Rust tests", flush=True)
    for target in sorted({m["rust_target"] for m in mappings}):
        result = run(["cargo","test","--locked","-p","rustycad-tkernel","--test",target,"--","--test-threads=1"],output/f"rust-{target}.log",env=env)
        passed = set(re.findall(r"^test (\S+) \.\.\. ok$",result,re.M))
        wanted = {name for mapped_target, name in rust_expected if mapped_target == target}
        if passed != wanted:
            raise RuntimeError(f"Rust tests missing, skipped or unmapped in {target}")
    if any(m["rust_target"] == "quantity_color" for m in mappings):
        # The named-color table is generated data; a stale copy would pass the Rust tests alone.
        run([sys.executable, ROOT/"tools/generate_quantity_color_table.py", "--check", "--source", source], output/"quantity-color-table-check.log", env=env)
    probe = output/"occt-probe"
    run([*common, ROOT/"tools/oracle/tkernel_probe.cxx", kernel, *rpaths, "-o", probe],output/"compile-probe.log",env=env)
    requests = corpus()
    (output/"inputs.txt").write_text(requests)
    oracle = run([probe],output/"occt-output.txt",input=requests,env=env)
    run(["cargo","build","--locked","-p","rustycad-tkernel","--example","oracle_probe"],output/"build-rust-probe.log",env=env)
    rust = run([ROOT/"target/debug/examples/oracle_probe"],output/"rust-output.txt",input=requests,env=env)
    if len(oracle.splitlines()) != len(requests.splitlines()):
        raise RuntimeError("Oracle returned an incomplete differential result")
    if oracle != rust:
        for i,(a,b) in enumerate(zip(oracle.splitlines(),rust.splitlines())):
            if a != b:
                raise RuntimeError(f"Differential mismatch for {requests.splitlines()[i]}: OCCT={a}; Rust={b}")
        raise RuntimeError("Differential result lengths differ")
    numeric_report = None
    if any(m["rust_target"] == "standard_strtod" for m in mappings):
        run([sys.executable, ROOT/"tools/verify_strtod.py", "--source", source], output/"strtod-verification.log", env=env)
        numeric_report = json.loads((output/"strtod/report.json").read_text())
    string_report = None
    if any(m["rust_target"] == "tcollection_ascii_string" for m in mappings):
        run([sys.executable, ROOT/"tools/verify_ascii_string.py", "--source", source], output/"ascii-string-verification.log", env=env)
        string_report = json.loads((output/"ascii-string/report.json").read_text())
    extended_report = None
    if any(m["rust_target"] == "tcollection_extended_string" for m in mappings):
        run([sys.executable, ROOT/"tools/verify_extended_string.py", "--source", source], output/"extended-string-verification.log", env=env)
        extended_report = json.loads((output/"extended-string/report.json").read_text())
    kdtree_report = None
    if any(m["rust_target"] == "ncollection_kdtree" for m in mappings):
        run([sys.executable, ROOT/"tools/verify_kdtree.py", "--source", source], output/"kdtree-verification.log", env=env)
        kdtree_report = json.loads((output/"kdtree/report.json").read_text())
    units_report = None
    if any(m["rust_target"] == "units_api" for m in mappings):
        run([sys.executable, ROOT/"tools/generate_units_data.py", ROOT/"tools/generate_quantity_color_table.py", "--check", "--source", source], output/"units-data-check.log", env=env)
        run([sys.executable, ROOT/"tools/verify_units.py", "--source", source], output/"units-verification.log", env=env)
        units_report = json.loads((output/"units/report.json").read_text())
    geometry_report = None
    if any(m["rust_target"] == "standard_dump" for m in mappings):
        run([sys.executable, ROOT/"tools/verify_geometry.py", "--source", source], output/"geometry-verification.log", env=env)
        geometry_report = json.loads((output/"geometry/report.json").read_text())
    files = sorted(set([p for p in (ROOT/"crates").rglob("*") if p.is_file()] + list((ROOT/"tools/oracle").glob("*.cxx")) + list((ROOT/"tools").glob("tkernel_*_corpus.py")) + [ROOT/"Cargo.toml", ROOT/"tools/verify_geometry.py", ROOT/"tools/verify_units.py", ROOT/"tools/generate_units_data.py", ROOT/"tools/generate_quantity_color_table.py", ROOT/"porting/units-data.json", ROOT/"tools/verify_kdtree.py", ROOT/"crates/tkernel/Cargo.toml", ROOT/"porting/parity.json", Path(__file__).resolve(), ROOT/"tools/oracle/tkernel_probe.cxx", ROOT/"tools/oracle/strtod_probe.cxx", ROOT/"tools/verify_strtod.py", ROOT/"tools/tkernel_mapping.py", ROOT/"tools/verify_ascii_string.py", ROOT/"tools/oracle/ascii_string_probe.cxx", ROOT/"tools/verify_extended_string.py", ROOT/"tools/oracle/extended_string_probe.cxx", ROOT/"Cargo.lock", ROOT/"rust-toolchain.toml"]))
    report = {"schema_version":1,"profile":"linux-gcc-c-locale","revision":inventory['revision'],
              "platform":platform.platform(),"python":platform.python_version(),
              "cmake_definitions":flags,
              "cmake":run(["cmake","--version"],output/"cmake.txt").splitlines()[0],
              "oracle_libraries_sha256":{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in libraries},
              "compiler":run([compiler,"--version"],output/"compiler.txt").splitlines()[0],
              "rustc":run(["rustc","--version"],output/"rustc.txt",env=env).strip(),
              "legacy_tests_passed":native_passed,"upstream_skips":native_skips,
              "declarations_mapped":len(mappings),"rust_translations_passed":sum(m["status"]=="translated" for m in mappings),
              "current_behavior_regressions_passed":len(native_skips),"rust_runtime_cases_passed":len(rust_expected),
              "differential_cases_passed":len(requests.splitlines()) + (numeric_report["cases_passed"] if numeric_report else 0) + (string_report["cases_passed"] if string_report else 0) + (extended_report["cases_passed"] if extended_report else 0) + (kdtree_report["cases_passed"] if kdtree_report else 0) + (units_report["cases_passed"] if units_report else 0) + (geometry_report["cases_passed"] if geometry_report else 0),
              "numeric_prefix_report":numeric_report,
              "ascii_string_report":string_report,
              "extended_string_report":extended_report,
              "kdtree_report":kdtree_report,
              "units_report":units_report,
              "geometry_report":geometry_report,
              "input_sha256":hashlib.sha256(requests.encode()).hexdigest(),
              "output_sha256":hashlib.sha256(oracle.encode()).hexdigest(),
              "source_hashes":{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in files},
              "toolkit_complete":False,"scope":"Mapped suites only; full TKernel and DRAW acceptance pending"}
    report_path.write_text(json.dumps(report,indent=2)+'\n')
    print(f"PASS: {len(mappings)} mapped declarations ({len(native_skips)} upstream skips with separate compatibility regressions) and {report['differential_cases_passed']} differential cases; report: {report_path.relative_to(ROOT)}")


if __name__ == '__main__':
    try:
        main()
    except (RuntimeError, ValueError, OSError, subprocess.TimeoutExpired) as error:
        print(f"error: {error}",file=sys.stderr)
        sys.exit(1)
