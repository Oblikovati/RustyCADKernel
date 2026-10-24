"""Preserve native red/green verdicts and compare every mesh word independently."""

from __future__ import annotations
import hashlib
import json
import os
import platform
import re
import shutil
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CASES = [
    "seam::cylinder_seam_copy_matches_pinned_native_order",
    "faces::cylinder_faces_match_pinned_native_mesh_hashes",
]
DEPENDENCIES = ["tkbrep", "tkernel", "tkg3d", "tkmath", "tkmesh", "tkprim", "tktopalgo"]


def sha256(path: Path) -> str:
    """Bind captured bytes; `sha256(Path('manifest.json'))`."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(product: Path, *arguments: str) -> str:
    """Read one exact checkout; `git(product, 'rev-parse', 'HEAD')`."""
    return subprocess.check_output(
        ["git", "-C", str(product), *arguments], text=True
    ).strip()


def validate_platform() -> None:
    """Reject accidental emulation/runner substitution; `validate_platform()`."""
    system, machine = platform.system(), platform.machine().lower()
    supported = (system == "Darwin" and machine in ("arm64", "aarch64")) or (
        system == "Windows" and machine in ("amd64", "x86_64")
    )
    if not supported:
        raise ValueError(
            f"runner {(system, machine)!r}: expected native macOS ARM64 or Windows x64"
        )


def validate_sources(product: Path, manifest: dict[str, object], fixed: bool) -> None:
    """Keep original tests and one selected helper immutable; `validate_sources(product, manifest, False)`."""
    expected = manifest["new_sha256" if fixed else "old_sha256"]
    path = product / str(manifest["changed_file"])
    if sha256(path) != expected:
        raise ValueError(f"helper {sha256(path)!r}: expected {expected!r}")
    for name, expected_test in dict(manifest["original_tests"]).items():
        actual = sha256(product / name)
        if actual != expected_test:
            raise ValueError(
                f"original test {name!r} hash {actual}: expected {expected_test}"
            )


def checkout_state(product: Path) -> dict[str, str]:
    """Preserve the actual offending paths before restoration; `checkout_state(product)`."""
    return {
        "porcelain": git(product, "status", "--porcelain", "--untracked-files=all"),
        "raw": git(product, "diff", "--raw"),
        "numstat": git(product, "diff", "--numstat"),
    }


def restore_tracked_blob(
    product: Path, revision: str, relative: str
) -> dict[str, object]:
    """Restore reviewed bytes without accepting generated content; `restore_tracked_blob(product, sha, path)`."""
    contents = regular_blob_contents(product, revision, relative)
    path = product / relative
    before = sha256(path) if path.exists() else None
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(contents)
    expected = hashlib.sha256(contents).hexdigest()
    if sha256(path) != expected:
        raise ValueError(
            f"restored {relative!r} hash {sha256(path)}: expected {expected}"
        )
    return {
        "path": relative,
        "before_sha256": before,
        "frozen_sha256": expected,
        "after_sha256": sha256(path),
    }


def regular_blob_contents(product: Path, revision: str, relative: str) -> bytes:
    """Reject symlinks/submodules before restoring source; `regular_blob_contents(product, sha, path)`."""
    entry = git(product, "ls-tree", revision, "--", relative).split()
    if not entry or entry[0] not in ("100644", "100755"):
        raise ValueError(
            f"tracked entry {entry!r}: expected a regular file at {relative!r}"
        )
    return subprocess.check_output(
        ["git", "-C", str(product), "show", f"{revision}:{relative}"]
    )


def frozen_index_entries(product: Path, revision: str, names: list[str]) -> bytes:
    """Refresh only reviewed regular entries; `frozen_index_entries(product, sha, paths)`."""
    wanted = {name.encode("utf-8") for name in names}
    tree = subprocess.check_output(
        ["git", "-C", str(product), "ls-tree", "-rz", revision]
    )
    entries: list[bytes] = []
    for row in tree.split(b"\0"):
        if not row:
            continue
        metadata, name = row.split(b"\t", 1)
        if name in wanted:
            mode, kind, object_id = metadata.split()
            if mode not in (b"100644", b"100755") or kind != b"blob":
                raise ValueError(f"entry {metadata!r}: expected a regular frozen blob")
            entries.append(mode + b" " + object_id + b"\t" + name + b"\0")
    if len(entries) != len(wanted):
        raise ValueError(
            f"entries {len(entries)}: expected {len(wanted)} frozen regular paths"
        )
    return b"".join(entries)


def refresh_frozen_index(
    product: Path, revision: str, names: list[str]
) -> dict[str, object]:
    """Clear stale EOL stat metadata without changing the index tree; `refresh_frozen_index(product, sha, paths)`."""
    expected = git(product, "rev-parse", revision + "^{tree}")
    before = git(product, "write-tree")
    if before != expected:
        raise ValueError(f"index tree {before}: expected frozen tree {expected}")
    entries = frozen_index_entries(product, revision, names)
    apply_frozen_entries(product, entries)
    after = git(product, "write-tree")
    if after != expected:
        raise ValueError(
            f"refreshed tree {after}: expected unchanged frozen tree {expected}"
        )
    return {
        "frozen_tree": expected,
        "before_tree": before,
        "after_tree": after,
        "refreshed_paths": len(names),
    }


def apply_frozen_entries(product: Path, entries: bytes) -> None:
    """Rewrite only existing frozen entry identities; `apply_frozen_entries(product, entries)`."""
    subprocess.run(
        ["git", "-C", str(product), "update-index", "-z", "--index-info"],
        input=entries,
        capture_output=True,
        check=True,
    )


def normalize_checkout(
    product: Path, output: Path, manifest: dict[str, object]
) -> None:
    """Restore only tracked drift, then retain a strict clean-tree gate; `normalize_checkout(product,out,manifest)`."""
    revision = str(manifest["product_revision"])
    if git(product, "rev-parse", "HEAD") != revision:
        raise ValueError(
            f"checkout {git(product, 'rev-parse', 'HEAD')}: expected {revision}"
        )
    before = checkout_state(product)
    (output / "checkout-before.json").write_text(json.dumps(before, indent=2) + "\n")
    git(product, "config", "core.autocrlf", "false")
    git(product, "config", "core.eol", "lf")
    modified = subprocess.check_output(
        ["git", "-C", str(product), "ls-files", "--modified", "-z"]
    )
    names = [name.decode("utf-8") for name in modified.split(b"\0") if name]
    restored = [restore_tracked_blob(product, revision, name) for name in names]
    index = refresh_frozen_index(product, revision, names)
    write_checkout_receipt(output, before, restored, index, checkout_state(product))


def write_checkout_receipt(
    output: Path,
    before: dict[str, str],
    restored: list[dict[str, object]],
    index: dict[str, object],
    after: dict[str, str],
) -> None:
    """Retain tree and byte identities at each stage; `write_checkout_receipt(out,before,files,index,after)`."""
    result = {
        "before": before,
        "restored": restored,
        "index_refresh": index,
        "after": after,
    }
    (output / "checkout-normalization.json").write_text(
        json.dumps(result, indent=2) + "\n"
    )


def validate_manifest(product: Path, manifest: dict[str, object]) -> None:
    """Admit only the reviewed baseline and assets; `validate_manifest(product, manifest)`."""
    actual = git(product, "rev-parse", "HEAD")
    if actual != manifest["product_revision"]:
        raise ValueError(
            f"product revision {actual}: expected {manifest['product_revision']}"
        )
    validate_assets(manifest)
    dirty = git(product, "status", "--porcelain")
    if dirty:
        raise ValueError(f"product checkout {dirty!r}: expected a clean exact baseline")
    validate_sources(product, manifest, False)


def validate_assets(manifest: dict[str, object]) -> None:
    """Bind each reviewed asset independently; `validate_assets(manifest)`."""
    assets = [
        ("selector-fix.patch", "patch_sha256"),
        ("portable-cylinder.expected", "independent_reference_sha256"),
        ("side-primitive-certificates.json", "primitive_certificate_sha256"),
        ("probe.lock", "lock_sha256"),
    ]
    expected = {name: str(manifest[field]) for name, field in assets}
    expected.update(dict(manifest["transport_assets"]))
    for name, digest in expected.items():
        if sha256(HERE / name) != digest:
            raise ValueError(
                f"asset {name!r} hash {sha256(HERE / name)}: expected {digest}"
            )


def stage_probe(product: Path, output: Path) -> Path:
    """Import unchanged original files into a bounded executable; `stage_probe(product, output)`."""
    probe = output / "probe"
    (probe / "src").mkdir(parents=True)
    (probe / "tests").mkdir()
    write_probe_manifest(product, probe)
    roots = product / "crates/martensite-tkmesh/tests"
    imports = [
        ("seam", "brep_mesh_cylinder_seam_order.rs"),
        ("faces", "brep_mesh_planar_face_discret.rs"),
    ]
    declarations = [
        f"#[path = {json.dumps((roots / name).as_posix())}]\nmod {module};\n"
        for module, name in imports
    ]
    (probe / "tests/original_contracts.rs").write_text("".join(declarations))
    shutil.copyfile(HERE / "cylinder_records.rs", probe / "src/main.rs")
    shutil.copyfile(HERE / "probe.lock", probe / "Cargo.lock")
    return probe


def write_probe_manifest(product: Path, probe: Path) -> None:
    """Match approved arithmetic profiling without adding features; `write_probe_manifest(product, probe)`."""
    dependencies = [
        f'martensite-{name} = {{ path = {json.dumps((product / "crates" / ("martensite-" + name)).as_posix())} }}'
        for name in DEPENDENCIES
    ]
    prefix = '[package]\nname="mesh263-capture"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[dependencies]\n'
    suffix = '\n[profile.dev.package.num-bigint]\nopt-level=3\n[[test]]\nname="original_contracts"\npath="tests/original_contracts.rs"\n'
    (probe / "Cargo.toml").write_text(prefix + "\n".join(dependencies) + suffix)


def capture(
    output: Path, label: str, command: list[str], environment: dict[str, str]
) -> dict[str, object]:
    """Retain streams before judging a verdict; `capture(out, 'build', argv, env)`."""
    stdout, stderr = output / (label + ".stdout.log"), output / (label + ".stderr.log")
    with stdout.open("xb") as out, stderr.open("xb") as err:
        result = subprocess.run(
            command,
            cwd=output / "probe",
            env=environment,
            stdout=out,
            stderr=err,
            timeout=1800,
        )
    return write_capture_receipt(output, label, command, result.returncode)


def write_capture_receipt(
    output: Path, label: str, command: list[str], status: int
) -> dict[str, object]:
    """Bind verdicts to the complete raw streams; `write_capture_receipt(out, 'test', argv, 101)`."""
    stdout, stderr = output / (label + ".stdout.log"), output / (label + ".stderr.log")
    receipt = {
        "command": command,
        "returncode": status,
        "stdout": stdout.name,
        "stderr": stderr.name,
        "stdout_sha256": sha256(stdout),
        "stderr_sha256": sha256(stderr),
    }
    (output / (label + ".json")).write_text(json.dumps(receipt, indent=2) + "\n")
    return receipt


def require_test_verdict(output: Path, receipt: dict[str, object], fixed: bool) -> None:
    """Require exactly one executed original regression; `require_test_verdict(out, receipt, True)`."""
    text = (output / str(receipt["stdout"])).read_text(errors="replace")
    pattern = r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;"
    matches = re.findall(pattern, text)
    expected = ("ok", "1", "0", "0") if fixed else ("FAILED", "0", "1", "0")
    status = 0 if fixed else 101
    if matches != [expected] or receipt["returncode"] != status:
        raise ValueError(
            f"test verdict {(matches, receipt['returncode'])!r}: expected {(expected, status)!r}"
        )


def capture_tests(
    output: Path, cargo: list[str], environment: dict[str, str], fixed: bool
) -> list[dict[str, object]]:
    """Keep both existing assertions at each phase; `capture_tests(out, cargo, env, False)`."""
    phase = "fixed" if fixed else "original"
    receipts: list[dict[str, object]] = []
    for index, case in enumerate(CASES):
        command = cargo + [
            "test",
            "--locked",
            "--test",
            "original_contracts",
            "--",
            case,
            "--exact",
            "--test-threads=1",
            "--nocapture",
        ]
        receipt = capture(output, f"{phase}-test-{index}", command, environment)
        require_test_verdict(output, receipt, fixed)
        receipts.append(receipt)
    return receipts


def capture_records(
    output: Path, cargo: list[str], environment: dict[str, str], phase: str
) -> list[dict[str, object]]:
    """Check every node/UV/index byte in two processes; `capture_records(out, cargo, env, 'original')`."""
    expected = (HERE / "portable-cylinder.expected").read_bytes()
    receipts: list[dict[str, object]] = []
    for repetition in (1, 2):
        receipt = capture(
            output,
            f"{phase}-records-{repetition}",
            cargo + ["run", "--quiet", "--locked"],
            environment,
        )
        actual = (output / str(receipt["stdout"])).read_bytes()
        if receipt["returncode"] != 0 or actual != expected:
            raise ValueError(
                f"{phase} repetition {repetition}: mesh {hashlib.sha256(actual).hexdigest()}, expected {hashlib.sha256(expected).hexdigest()}"
            )
        receipts.append(receipt)
    return receipts


def apply_checked_fix(product: Path, manifest: dict[str, object]) -> None:
    """Apply only the reviewed test selector; `apply_checked_fix(product, manifest)`."""
    patch = str(HERE / "selector-fix.patch")
    subprocess.run(["git", "-C", str(product), "apply", "--check", patch], check=True)
    subprocess.run(["git", "-C", str(product), "apply", patch], check=True)
    if git(product, "diff", "--name-only") != manifest["changed_file"]:
        raise ValueError(
            "patch touched unexpected files: expected only the test snapshot selector"
        )
    validate_sources(product, manifest, True)


def run_contract(
    product: Path, output: Path, manifest: dict[str, object]
) -> dict[str, object]:
    """Prove original failures, unchanged geometry and corrected assertions; `run_contract(product, out, manifest)`."""
    stage_probe(product, output)
    environment = dict(
        os.environ, CARGO_TARGET_DIR=str(output / "build"), CARGO_TERM_COLOR="never"
    )
    cargo = ["cargo", "+1.99.0"]
    return capture_phases(product, output, manifest, cargo, environment)


def capture_phases(
    product: Path,
    output: Path,
    manifest: dict[str, object],
    cargo: list[str],
    environment: dict[str, str],
) -> dict[str, object]:
    """Run one immutable baseline and one checked selector; `capture_phases(product,out,manifest,cargo,env)`."""
    features = require_default_features(output, cargo, environment)
    original_records = capture_records(output, cargo, environment, "original")
    original_tests = capture_tests(output, cargo, environment, False)
    apply_checked_fix(product, manifest)
    fixed_records = capture_records(output, cargo, environment, "fixed")
    fixed_tests = capture_tests(output, cargo, environment, True)
    return {
        "features": features,
        "original_records": original_records,
        "original_tests": original_tests,
        "fixed_records": fixed_records,
        "fixed_tests": fixed_tests,
    }


def require_default_features(
    output: Path, cargo: list[str], environment: dict[str, str]
) -> dict[str, object]:
    """Exclude verification-only math replay from this release contract; `require_default_features(out,cargo,env)`."""
    command = cargo + ["tree", "--locked", "-e", "features", "-i", "martensite-cmath"]
    receipt = capture(output, "default-features", command, environment)
    text = (output / str(receipt["stdout"])).read_text()
    if receipt["returncode"] != 0 or any(
        name in text for name in ("host-libm", "libm-attribution")
    ):
        raise ValueError(
            "feature tree: expected default portable math without replay/attribution"
        )
    return receipt


def main() -> None:
    """Capture the exact native runner and retain any terminal failure; `main()`."""
    product, output = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
    output.mkdir(parents=True)
    manifest = json.loads((HERE / "manifest.json").read_text())
    validate_platform()
    normalize_checkout(product, output, manifest)
    validate_manifest(product, manifest)
    result = run_contract(product, output, manifest)
    receipt = {
        "platform": platform.platform(),
        "machine": platform.machine(),
        "product_revision": git(product, "rev-parse", "HEAD"),
        "transport_revision": git(HERE, "rev-parse", "HEAD"),
        "manifest": manifest,
        "rust": subprocess.check_output(["rustc", "+1.99.0", "-Vv"], text=True),
        "success": True,
        "checks": result,
    }
    (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        destination = Path(sys.argv[2]).resolve()
        destination.mkdir(parents=True, exist_ok=True)
        failure = {"success": False, "class": type(error).__name__, "cause": str(error)}
        (destination / "failure.json").write_text(json.dumps(failure, indent=2) + "\n")
        raise
