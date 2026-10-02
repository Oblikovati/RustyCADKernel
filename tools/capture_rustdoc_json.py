#!/usr/bin/env python3
"""Capture and verify pinned rustdoc JSON for every workspace library crate."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
TOOLCHAIN_CONFIG = ROOT / "porting/api-proof-toolchain.json"
DEFAULT_OUTPUT = ROOT / "target/api-proof/rustdoc-json"
FINGERPRINT_FILES = (
    ROOT / "Cargo.toml",
    ROOT / "Cargo.lock",
    ROOT / "rust-toolchain.toml",
    ROOT / "porting/api-proof-toolchain.json",
    ROOT / "porting/api-header-manifest.json",
    pathlib.Path(__file__).resolve(),
)
ENVIRONMENT_KEYS = (
    "PATH",
    "CARGO_HOME",
    "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "RUSTDOCFLAGS",
    "CARGO_BUILD_TARGET",
    "RUSTC_BOOTSTRAP",
    "RUSTC",
    "RUSTDOC",
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "CC",
    "CXX",
    "AR",
    "CFLAGS",
    "CXXFLAGS",
    "LDFLAGS",
)


@dataclass(frozen=True)
class LibraryCrate:
    package_id: str
    package_name: str
    manifest: pathlib.Path
    target_name: str
    source_root: pathlib.Path
    features: tuple[str, ...]


def run_checked(command: list[str], env: dict[str, str]) -> str:
    result = subprocess.run(command, cwd=ROOT, env=env, text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(f"command failed ({result.returncode}): {' '.join(command)}\n{result.stderr}")
    return result.stdout


def cargo_environment() -> dict[str, str]:
    env = os.environ.copy()
    cargo_home = pathlib.Path(env.get("CARGO_HOME", pathlib.Path.home() / ".cargo"))
    env["PATH"] = f"{cargo_home / 'bin'}{os.pathsep}{env.get('PATH', '')}"
    return env


def rust_toolchain() -> tuple[str, dict[str, Any]]:
    config = json.loads(TOOLCHAIN_CONFIG.read_text(encoding="utf-8"))
    nightly = config["rust_nightly"]
    env = cargo_environment()
    versions = {
        "rustc": run_checked(["rustc", f"+{nightly}", "-vV"], env),
        "rustdoc": run_checked(["rustdoc", f"+{nightly}", "--version", "--verbose"], env),
        "cargo": run_checked(["cargo", f"+{nightly}", "--version"], env),
    }
    return nightly, versions


def workspace_metadata(nightly: str, env: dict[str, str]) -> tuple[dict[str, Any], str]:
    output = run_checked(
        ["cargo", f"+{nightly}", "metadata", "--locked", "--no-deps", "--format-version", "1"], env
    )
    return json.loads(output), output


def workspace_library_crates(metadata: dict[str, Any]) -> list[LibraryCrate]:
    members = set(metadata["workspace_members"])
    packages = [package for package in metadata["packages"] if package["id"] in members]
    if len(packages) != len(members):
        raise ValueError("Cargo metadata omitted one or more declared workspace members")
    crates = [library_crate(package) for package in packages]
    target_names = [crate.target_name for crate in crates]
    if len(target_names) != len(set(target_names)):
        raise ValueError("workspace library target names are not unique")
    return sorted(crates, key=lambda crate: crate.package_name)


def library_crate(package: dict[str, Any]) -> LibraryCrate:
    targets = [target for target in package["targets"] if "lib" in target["kind"]]
    if len(targets) != 1:
        raise ValueError(f"workspace package {package['name']} has {len(targets)} library targets; expected one")
    target = targets[0]
    manifest = pathlib.Path(package["manifest_path"]).resolve()
    return LibraryCrate(
        package_id=package["id"],
        package_name=package["name"],
        manifest=manifest,
        target_name=target["name"],
        source_root=manifest.parent,
        features=tuple(sorted(package.get("features", {}))),
    )


def file_digest(path: pathlib.Path) -> str:
    """Return the content digest used in artifact and input fingerprints.

    Example: `file_digest(Path("Cargo.lock"))` produces a SHA-256 hex string.
    """
    return hashlib.sha256(path.read_bytes()).hexdigest()


def file_identity(path: pathlib.Path, base: pathlib.Path) -> str:
    resolved = path.resolve()
    try:
        return resolved.relative_to(base).as_posix()
    except ValueError:
        return resolved.as_posix()


def digest_paths(paths: list[pathlib.Path], base: pathlib.Path) -> str:
    digest = hashlib.sha256()
    for path in sorted(set(paths)):
        digest.update(file_identity(path, base).encode())
        digest.update(b"\0")
        digest.update(file_digest(path).encode())
        digest.update(b"\n")
    return digest.hexdigest()


def workspace_source_paths() -> list[pathlib.Path]:
    """List package inputs, including build scripts and included data.

    The broad file set keeps `include!` and build-script input changes visible.
    """
    return [path for path in (ROOT / "crates").rglob("*") if path.is_file() and ".git" not in path.parts]


def config_paths() -> list[pathlib.Path]:
    candidates = [
        parent / ".cargo" / filename
        for parent in (ROOT, *ROOT.parents)
        for filename in ("config.toml", "config")
    ]
    cargo_home = pathlib.Path(os.environ.get("CARGO_HOME", pathlib.Path.home() / ".cargo"))
    candidates.extend((cargo_home / filename for filename in ("config.toml", "config")))
    return [path for path in candidates if path.is_file()]


def fingerprint_context(metadata_output: str, versions: dict[str, Any]) -> dict[str, Any]:
    source_paths = workspace_source_paths()
    fixed_paths = [path for path in FINGERPRINT_FILES if path.is_file()]
    fixed_paths.extend(config_paths())
    return {
        "workspace_revision": run_checked(["git", "rev-parse", "HEAD"], cargo_environment()).strip(),
        "toolchain_config_sha256": file_digest(TOOLCHAIN_CONFIG),
        "cargo_metadata_sha256": hashlib.sha256(metadata_output.encode()).hexdigest(),
        "workspace_inputs_sha256": digest_paths(source_paths + fixed_paths, ROOT),
        "workspace_file_count": len(source_paths),
        "environment": {key: os.environ[key] for key in ENVIRONMENT_KEYS if key in os.environ},
        "tool_versions": versions,
    }


def validate_rustdoc_json(document: Any, crate_name: str) -> tuple[int, int]:
    if not isinstance(document, dict):
        raise ValueError(f"rustdoc output for {crate_name} is not a JSON object")
    required = {"root", "index", "paths", "format_version"}
    missing = sorted(required - document.keys())
    if missing:
        raise ValueError(f"rustdoc output for {crate_name} lacks fields: {', '.join(missing)}")
    if not isinstance(document["index"], dict) or not isinstance(document["paths"], dict):
        raise ValueError(f"rustdoc output for {crate_name} has an invalid index or paths table")
    root = document["index"].get(str(document["root"]))
    if not isinstance(root, dict) or root.get("name") != crate_name:
        raise ValueError(f"rustdoc output root does not match crate {crate_name}")
    if not isinstance(document["format_version"], int):
        raise ValueError(f"rustdoc output for {crate_name} has no integer format version")
    return document["format_version"], len(document["index"])


def capture_command(nightly: str, crate: LibraryCrate, build_dir: pathlib.Path) -> list[str]:
    return [
        "cargo", f"+{nightly}", "rustdoc", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"),
        "--package", crate.package_name, "--lib", "--all-features", "--target-dir", str(build_dir),
        "-Z", "unstable-options", "--", "--output-format", "json", "-Z", "unstable-options",
    ]


def capture_crate(nightly: str, crate: LibraryCrate, build_dir: pathlib.Path, stage: pathlib.Path,
                  env: dict[str, str]) -> dict[str, Any]:
    command = capture_command(nightly, crate, build_dir)
    print(f"Capturing rustdoc JSON for {crate.package_name}", flush=True)
    run_checked(command, env)
    generated = build_dir / "doc" / f"{crate.target_name}.json"
    if not generated.is_file():
        raise FileNotFoundError(f"rustdoc did not produce the expected artifact for {crate.package_name}: {generated}")
    document = json.loads(generated.read_text(encoding="utf-8"))
    format_version, item_count = validate_rustdoc_json(document, crate.target_name)
    relative = pathlib.Path("crates") / f"{crate.target_name}.json"
    destination = stage / relative
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(generated, destination)
    return crate_artifact_record(crate, relative, destination, format_version, item_count)


def crate_artifact_record(crate: LibraryCrate, relative: pathlib.Path, output: pathlib.Path,
                          format_version: int, item_count: int) -> dict[str, Any]:
    package_paths = [path for path in crate.source_root.rglob("*") if path.is_file()]
    return {
        "package": crate.package_name,
        "package_id": crate.package_id,
        "library_target": crate.target_name,
        "manifest_path": crate.manifest.relative_to(ROOT).as_posix(),
        "available_features": list(crate.features),
        "feature_selection": "all-features",
        "package_inputs_sha256": digest_paths(package_paths, ROOT),
        "artifact": relative.as_posix(),
        "artifact_sha256": file_digest(output),
        "artifact_bytes": output.stat().st_size,
        "rustdoc_format_version": format_version,
        "indexed_items": item_count,
    }


def capture_manifest(context: dict[str, Any], records: list[dict[str, Any]]) -> dict[str, Any]:
    return {
        "schema_version": 1,
        "raw_capture_complete": True,
        "normalized_public_inventory": False,
        "workspace_library_count": len(records),
        "feature_policy": "all-features per workspace library package",
        "cargo_arguments": [
            "rustdoc", "--locked", "--package {package}", "--lib", "--all-features",
            "-Z unstable-options", "-- --output-format json -Z unstable-options",
        ],
        "rustdoc_json": records,
        "inputs": context,
    }


def validate_capture(directory: pathlib.Path, expected: list[LibraryCrate], context: dict[str, Any]) -> None:
    manifest = json.loads((directory / "capture-manifest.json").read_text(encoding="utf-8"))
    if manifest.get("normalized_public_inventory") is not False:
        raise ValueError("raw rustdoc capture must not claim normalized inventory completeness")
    if manifest.get("inputs") != context:
        raise ValueError("workspace inputs changed during rustdoc capture")
    records = manifest.get("rustdoc_json", [])
    if manifest.get("raw_capture_complete") is not True or len(records) != len(expected):
        raise ValueError("rustdoc capture is incomplete")
    expected_names = {crate.target_name for crate in expected}
    if {record.get("library_target") for record in records} != expected_names:
        raise ValueError("rustdoc capture is missing or duplicates a workspace library")
    for record in records:
        validate_artifact(directory, record)


def validate_artifact(directory: pathlib.Path, record: dict[str, Any]) -> None:
    artifact = (directory / record["artifact"]).resolve()
    try:
        artifact.relative_to(directory.resolve())
    except ValueError as error:
        raise ValueError(f"rustdoc artifact escapes the capture directory: {artifact}") from error
    if not artifact.is_file() or file_digest(artifact) != record["artifact_sha256"]:
        raise ValueError(f"rustdoc artifact is missing or changed: {artifact}")
    if artifact.stat().st_size != record["artifact_bytes"]:
        raise ValueError(f"rustdoc artifact size differs from its capture record: {artifact}")
    document = json.loads(artifact.read_text(encoding="utf-8"))
    format_version, item_count = validate_rustdoc_json(document, record["library_target"])
    if (format_version, item_count) != (record["rustdoc_format_version"], record["indexed_items"]):
        raise ValueError(f"rustdoc artifact metadata does not match its capture record: {artifact}")


def safe_output_path(path: pathlib.Path) -> pathlib.Path:
    output = path.resolve()
    managed_root = (ROOT / "target/api-proof").resolve()
    if managed_root not in output.parents:
        raise ValueError(f"capture output must be under {managed_root}, got {output}")
    return output


def capture_all(output: pathlib.Path) -> None:
    config = json.loads(TOOLCHAIN_CONFIG.read_text(encoding="utf-8"))
    nightly, versions = rust_toolchain()
    if nightly != config["rust_nightly"]:
        raise ValueError("resolved Rust toolchain differs from the pinned config")
    env = cargo_environment()
    metadata, raw_metadata = workspace_metadata(nightly, env)
    crates = workspace_library_crates(metadata)
    context = fingerprint_context(raw_metadata, versions)
    output.parent.mkdir(parents=True, exist_ok=True)
    stage = pathlib.Path(tempfile.mkdtemp(prefix=".rustdoc-capture-", dir=output.parent))
    try:
        generate_capture_contents(stage, nightly, crates, env, context, raw_metadata, versions)
        publish_capture(stage, output)
    finally:
        if stage.exists():
            shutil.rmtree(stage)


def generate_capture_contents(stage: pathlib.Path, nightly: str, crates: list[LibraryCrate],
                              env: dict[str, str], context: dict[str, Any],
                              metadata_output: str, versions: dict[str, Any]) -> None:
    build_dir = stage / "cargo-target"
    records = [capture_crate(nightly, crate, build_dir, stage, env) for crate in crates]
    shutil.rmtree(build_dir)
    manifest = capture_manifest(context, records)
    (stage / "capture-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    if fingerprint_context(metadata_output, versions) != context:
        raise ValueError("workspace inputs changed during rustdoc capture")
    validate_capture(stage, crates, context)


def publish_capture(stage: pathlib.Path, output: pathlib.Path) -> None:
    if output.exists():
        if not (output / "capture-manifest.json").is_file():
            raise FileExistsError(f"refusing to replace unmanaged rustdoc output: {output}")
        shutil.rmtree(output)
    stage.rename(output)


def verify_all(output: pathlib.Path) -> None:
    config = json.loads(TOOLCHAIN_CONFIG.read_text(encoding="utf-8"))
    nightly, versions = rust_toolchain()
    env = cargo_environment()
    metadata, raw_metadata = workspace_metadata(nightly, env)
    crates = workspace_library_crates(metadata)
    context = fingerprint_context(raw_metadata, versions)
    validate_capture(output, crates, context)
    print(f"Verified {len(crates)} pinned rustdoc JSON crate artifacts; inventory normalization remains false.")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("capture", "verify"))
    parser.add_argument("--output", type=pathlib.Path, default=DEFAULT_OUTPUT)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    output = safe_output_path(args.output if args.output.is_absolute() else ROOT / args.output)
    (capture_all if args.command == "capture" else verify_all)(output)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, KeyError, RuntimeError, json.JSONDecodeError) as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(2)
