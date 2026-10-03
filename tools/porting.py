#!/usr/bin/env python3
"""Reproduce the pinned OCCT source inventory; never infer API parity from files."""
import argparse
import collections
import json
from pathlib import Path
import re
import subprocess
import sys
from api_inventory_status import compiler_status_lines, current_workspace_digest

ROOT = Path(__file__).resolve().parents[1]
BASELINE = ROOT / "porting/baseline.json"
INVENTORY = ROOT / "porting/inventory.json"


def git(source, *args):
    return subprocess.check_output(["git", "-C", str(source), *args], text=True).strip()


def cmake_list(path, variable):
    text = re.sub(r"#[^\n]*", "", path.read_text())
    match = re.search(r"set\s*\(\s*" + re.escape(variable) + r"\s+([^)]*)\)", text)
    if not match:
        raise ValueError(f"Missing CMake list {variable} in {path}")
    tokens = match.group(1).split()
    if any(not re.fullmatch(r"[A-Za-z0-9_./]+", token) for token in tokens):
        raise ValueError(f"Unsupported CMake syntax in {path}; update the inventory reader")
    return tokens


def snapshot(source):
    baseline = json.loads(BASELINE.read_text())
    revision = git(source, "rev-parse", "HEAD")
    if revision != baseline["revision"]:
        raise ValueError(f"Expected OCCT {baseline['revision']}, found {revision}")
    if git(source, "status", "--porcelain", "--untracked-files=normal"):
        raise ValueError("OCCT source must be clean, including untracked files; use an out-of-tree build")
    paths = git(source, "ls-files").splitlines()
    modules = cmake_list(source / "src/MODULES.cmake", "OCCT_LIST_OF_MODULES")
    toolkits = []
    for module in modules:
        for name in cmake_list(source / f"src/{module}/TOOLKITS.cmake", f"OCCT_{module}_LIST_OF_TOOLKITS"):
            prefix = f"src/{module}/{name}/"
            directory = source / prefix
            packages = cmake_list(directory / "PACKAGES.cmake", f"OCCT_{name}_LIST_OF_PACKAGES")
            dependencies = cmake_list(directory / "EXTERNLIB.cmake", f"OCCT_{name}_EXTERNAL_LIBS")
            members = [path for path in paths if path.startswith(prefix)]
            headers = [p for p in members if Path(p).suffix in {".h", ".hxx", ".hpp", ".lxx", ".gxx", ".pxx"}]
            tests = [p for p in members if "/GTests/" in p and p.endswith(".cxx")]
            declarations = []
            for path in tests:
                # Discovery candidates only: comments, macros, conditional builds,
                # parameterized and typed tests require runtime enumeration later.
                for match in re.finditer(r"\b(TEST|TEST_F|TEST_P|TYPED_TEST|TYPED_TEST_P)\s*\(\s*(\w+)\s*,\s*(\w+)\s*\)", (source / path).read_text()):
                    declarations.append({"source": path, "macro": match[1], "name": f"{match[2]}.{match[3]}"})
            toolkits.append({"module": module, "name": name, "packages": packages,
                             "dependencies": dependencies,
                             "package_sources": [(directory / p).resolve().relative_to(source).as_posix() for p in packages],
                             "header_candidates": headers,
                             "gtest_sources": tests, "gtest_declaration_candidates": declarations})
    listed = {f"src/{t['module']}/{t['name']}/PACKAGES.cmake" for t in toolkits}
    discovered = {p for p in paths if re.fullmatch(r"src/[^/]+/[^/]+/PACKAGES.cmake", p)}
    if listed != discovered:
        raise ValueError(f"Unaccounted toolkit definitions: {sorted(listed ^ discovered)}")
    test_files = [p for p in paths if p.startswith("tests/")]
    # Preserve every test file, including begin/end, parse rules, lists and data.
    # Case classification is provisional until DRAW enumerates its suites.
    support_names = {"begin", "end", "grids.list", "parse.rules"}
    draw_candidates = [p for p in test_files if len(Path(p).parts) == 4
                       and Path(p).name not in support_names]
    return {"schema_version": 1, "revision": revision, "modules": modules,
            "toolkits": toolkits, "upstream_test_files": test_files,
            "draw_case_candidates": draw_candidates,
            "fixture_files": [p for p in paths if p.startswith(("data/", "resources/"))]}


def status():
    inventory = json.loads(INVENTORY.read_text())
    ledger = json.loads((ROOT / "porting/parity.json").read_text())
    counts = collections.Counter(t["module"] for t in inventory["toolkits"])
    for module, count in counts.items():
        print(f"{module}: {count} toolkits")
    print(f"Total: {len(inventory['toolkits'])} toolkits; "
          f"{sum(len(t['packages']) for t in inventory['toolkits'])} packages")
    print(f"Header candidates: {sum(len(t['header_candidates']) for t in inventory['toolkits'])}")
    print(f"GoogleTest source files: {sum(len(t['gtest_sources']) for t in inventory['toolkits'])}")
    print(f"GoogleTest declaration candidates: {sum(len(t['gtest_declaration_candidates']) for t in inventory['toolkits'])}")
    print(f"DRAW case candidates: {len(inventory['draw_case_candidates'])}")
    print(f"All regression files (including support): {len(inventory['upstream_test_files'])}")
    print(f"API mappings: {len(ledger['api'])}; test mappings: {len(ledger['tests'])}")
    for toolkit in inventory["toolkits"]:
        mapped = [row for row in ledger["tests"] if row["toolkit"] == toolkit["name"]]
        if mapped:
            print(f"{toolkit['name']}: {len(mapped)}/{len(toolkit['gtest_declaration_candidates'])} GoogleTest declarations mapped (source discovery, not full runtime/DRAW coverage)")
    for line in compiler_status_lines(ROOT, current_workspace_digest()):
        print(line)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["snapshot", "check", "status"])
    parser.add_argument("--source", type=Path, default=ROOT / "../OCCT")
    args = parser.parse_args()
    try:
        if args.command == "status":
            status()
        else:
            rendered = json.dumps(snapshot(args.source.resolve()), indent=2) + "\n"
            if args.command == "snapshot":
                INVENTORY.write_text(rendered)
                print(f"Wrote {INVENTORY.relative_to(ROOT)}")
            elif not INVENTORY.exists() or INVENTORY.read_text() != rendered:
                raise ValueError("Inventory differs; inspect upstream changes and regenerate explicitly")
            else:
                print("Pinned OCCT source inventory matches")
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
