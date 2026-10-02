#!/usr/bin/env python3
"""Extract the pinned OCCT CMake install-header source manifest.

This is the source boundary for the later compiler-backed C++ declaration inventory.
It does not count declarations and does not prove API coverage.
"""
from __future__ import annotations

import argparse
from collections import Counter
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
HEADER_SUFFIXES = {".h", ".hxx", ".lxx"}
LIST_PATTERN = re.compile(r"set\s*\(\s*([A-Za-z0-9_]+)\s+([^)]*)\)", re.S)


def git(source: Path, *arguments: str) -> str:
    """Read a Git fact about source; git(source, 'rev-parse', 'HEAD')."""
    result = subprocess.check_output(["git", "-C", str(source), *arguments], text=True)
    return result.strip()


def cmake_list(path: Path, variable: str) -> list[str]:
    """Read one literal CMake `set(NAME ...)` list; cmake_list(path, 'OCCT_MODULES')."""
    text = re.sub(r"#[^\n]*", "", path.read_text())
    match = next((item for item in LIST_PATTERN.finditer(text) if item.group(1) == variable), None)
    if match is None:
        raise ValueError(f"{path}: missing literal list {variable}")
    tokens = match.group(2).split()
    if any("$" in token or "(" in token for token in tokens):
        raise ValueError(f"{path}: unsupported CMake expression in {variable}")
    return tokens


def sha256(path: Path) -> str:
    """Hash a source file; sha256(path) returns the lowercase SHA-256 hex digest."""
    digest = hashlib.sha256(path.read_bytes())
    return digest.hexdigest()


def package_headers(toolkit_dir: Path, package: str) -> list[Path]:
    """Resolve CMake-listed installed headers for one package; package_headers(root, dir, 'gp')."""
    directory = toolkit_dir if package == "@toolkit" else (toolkit_dir / package).resolve()
    file_list = directory / "FILES.cmake"
    owner = directory.name if package == "@toolkit" else package
    owner = Path(owner).name
    files = cmake_list(file_list, f"OCCT_{owner}_FILES")
    paths = [directory / re.sub(r"^[^:]+:+", "", item) for item in files]
    missing = [str(path) for path in paths if not path.is_file()]
    if missing:
        raise ValueError(f"{file_list}: listed files missing from source: {missing[:5]}")
    return [path for path in paths if path.suffix in HEADER_SUFFIXES]


def collect(source: Path) -> dict[str, object]:
    """Collect every module-listed toolkit header selected by OCCT's install macro."""
    revision = validate_source(source)
    modules = cmake_list(source / "src/MODULES.cmake", "OCCT_LIST_OF_MODULES")
    toolkits, header_rows, dependencies = collect_modules(source, modules)
    return make_manifest(source, revision, modules, toolkits, header_rows, dependencies)


def validate_source(source: Path) -> str:
    """Require the clean OCCT checkout pinned by the repository baseline."""
    baseline = json.loads((ROOT / "porting/baseline.json").read_text())
    revision = git(source, "rev-parse", "HEAD")
    if revision != baseline["revision"]:
        raise ValueError(f"expected OCCT {baseline['revision']}, found {revision}")
    if git(source, "status", "--porcelain", "--untracked-files=normal"):
        raise ValueError("OCCT checkout must be clean; build artifacts must stay out of tree")

    return revision


def collect_modules(
    source: Path, modules: list[str]
) -> tuple[list[dict[str, object]], dict[str, dict[str, object]], list[dict[str, object]]]:
    """Collect toolkit header ownership and external dependency provenance."""
    toolkits = []
    header_rows: dict[str, dict[str, object]] = {}
    dependencies = []
    for module in modules:
        module_file = source / f"src/{module}/TOOLKITS.cmake"
        names = cmake_list(module_file, f"OCCT_{module}_LIST_OF_TOOLKITS")
        for toolkit in names:
            manifest, headers, external = collect_toolkit(source, module, toolkit)
            toolkits.append(manifest)
            dependencies.append(external)
            for header in headers:
                record_header_owner(header_rows, **header)

    return toolkits, header_rows, dependencies


def collect_toolkit(
    source: Path, module: str, toolkit: str
) -> tuple[dict[str, object], list[dict[str, object]], dict[str, object]]:
    """Collect one CMake-listed toolkit and each package's installed header candidates."""
    directory = source / f"src/{module}/{toolkit}"
    packages = cmake_list(directory / "PACKAGES.cmake", f"OCCT_{toolkit}_LIST_OF_PACKAGES")
    dependency_names = cmake_list(directory / "EXTERNLIB.cmake", f"OCCT_{toolkit}_EXTERNAL_LIBS")
    paths = package_headers(directory, "@toolkit")
    for package in packages:
        paths.extend(package_headers(directory, package))
    headers = [header_owner(source, path, directory, module, toolkit) for path in paths]
    return ({"module": module, "name": toolkit, "packages": packages}, headers,
            {"toolkit": toolkit, "names": dependency_names})


def header_owner(
    source: Path, path: Path, toolkit_dir: Path, module: str, toolkit: str
) -> dict[str, object]:
    """Return source hash, path, and the toolkit/package identity for one header."""
    return {"relative": path.relative_to(source).as_posix(), "path": path,
            "module": module, "toolkit": toolkit,
            "package": "@toolkit" if path.parent == toolkit_dir else path.parent.name}


def make_manifest(
    source: Path,
    revision: str,
    modules: list[str],
    toolkits: list[dict[str, object]],
    header_rows: dict[str, dict[str, object]],
    dependencies: list[dict[str, object]],
) -> dict[str, object]:
    """Render the versioned header manifest and explicit API scope exclusions."""
    validate_install_names(header_rows)
    manifest = {"schema": "occt-installed-public-header-manifest-v1",
                "authority": "CMake FILES.cmake lists and COLLECT_AND_INSTALL_OCCT_HEADER_FILES",
                "revision": revision}
    manifest.update(manifest_contents(source, modules, toolkits, header_rows, dependencies))
    manifest.update(scope_metadata())
    return manifest


def validate_install_names(header_rows: dict[str, dict[str, object]]) -> None:
    """Fail if CMake's flat include directory would receive two distinct headers by name."""
    counts = Counter(Path(path).name for path in header_rows)
    duplicates = sorted(name for name, count in counts.items() if count > 1)
    if duplicates:
        raise ValueError(f"duplicate CMake-installed header basenames: {duplicates[:10]}")
    if "Standard_Version.hxx" in counts:
        raise ValueError("generated Standard_Version.hxx conflicts with a source header")


def manifest_contents(
    source: Path,
    modules: list[str],
    toolkits: list[dict[str, object]],
    header_rows: dict[str, dict[str, object]],
    dependencies: list[dict[str, object]],
) -> dict[str, object]:
    """Build CMake-derived content fields of the manifest."""
    return {"modules": modules, "toolkits": toolkits,
            "headers": [{"path": path, **header_rows[path]} for path in sorted(header_rows)],
            "generated_headers": generated_header(source), "external_dependencies": dependencies}


def scope_metadata() -> dict[str, object]:
    """Keep this source-list artifact explicitly short of declaration and coverage proof."""
    return {
        "exclusion_policy": {
            "scope": "OCCT-owned declarations in module-listed toolkits and CMake-installed headers",
            "exclude": ["third-party declarations without an OCCT-owned installed header"],
            "include": ["OCCT wrappers and adapters that depend on external C libraries"],
            "note": "EXTERNLIB names are recorded for provenance, not used to exclude OCCT source APIs",
        },
        "declaration_inventory_complete": False,
        "coverage_proof": False,
    }


def generated_header(source: Path) -> list[dict[str, str]]:
    """Describe the generated public version header and its template provenance."""
    template = source / "adm/templates/Standard_Version.hxx.in"
    return [{"installed_name": "Standard_Version.hxx",
             "source_template": template.relative_to(source).as_posix(),
             "template_sha256": sha256(template),
             "generator": "OCCT_CONFIGURE_AND_INSTALL in CMakeLists.txt"}]


def record_header_owner(
    header_rows: dict[str, dict[str, object]],
    relative: str,
    path: Path,
    module: str,
    toolkit: str,
    package: str,
) -> None:
    """Record a CMake-installed header and its owning toolkit(s)."""
    owner = {"module": module, "toolkit": toolkit, "package": package}
    if relative not in header_rows:
        header_rows[relative] = {"sha256": sha256(path), "owners": [owner]}
        return
    record = header_rows[relative]
    if record["sha256"] != sha256(path):
        raise ValueError(f"header path has conflicting source contents: {relative}")
    if owner not in record["owners"]:
        record["owners"].append(owner)


def main() -> int:
    """Write a reproducible source manifest; no declaration-coverage claim is made."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=ROOT / "../OCCT")
    parser.add_argument("--output", type=Path, default=ROOT / "porting/api-header-manifest.json")
    parser.add_argument("--check", action="store_true", help="fail if the checked-in manifest is stale")
    args = parser.parse_args()
    try:
        rendered = json.dumps(collect(args.source.resolve()), indent=2) + "\n"
        if args.check:
            if not args.output.is_file() or args.output.read_text() != rendered:
                raise ValueError(f"{args.output} is stale; regenerate and review the header source set")
        else:
            args.output.write_text(rendered)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    print(f"{'Verified' if args.check else 'Wrote'} CMake-installed OCCT header manifest: {args.output}")
    print("Declaration inventory and API coverage remain unproven")
    return 0


if __name__ == "__main__":
    sys.exit(main())
