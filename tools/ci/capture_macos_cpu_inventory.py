#!/usr/bin/env python3
"""Retain unaccepted native compiler evidence using the product's checks-only gate."""
from __future__ import annotations

import hashlib
import json
import os
import platform
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Mapping

PRODUCT_SHA = "35dcacdaf9d0df729cfa90566933945fae047f21"
BASE_SHA = "681283b9bc5de2088ea9aa340506ea3b127b6744"
TOOLCHAIN = "1.99.0"
INVENTORY_NAMES = ("latest.json", "compiler.stdout.jsonl", "compiler.stderr.txt")
NATIVE_TARGETS = {"arm64": "aarch64-apple-darwin", "x86_64": "x86_64-apple-darwin"}
INVENTORY_OS_ENVIRONMENT_KEYS = ("PATH", "HOME", "USER", "LOGNAME", "TMPDIR", "LANG", "LC_ALL", "LC_CTYPE")


def inventory_capture_environment(source: Mapping[str, str], target: Path) -> dict[str, str]:
    """Forward native OS necessities, not arbitrary credentials/config; call before spawning."""
    result = {key: source[key] for key in INVENTORY_OS_ENVIRONMENT_KEYS if key in source}
    result.update(CARGO_TARGET_DIR=str(target), CARGO_HOME=str(target.with_name(target.name + "-cargo-home")))
    result.update(CARGO_INCREMENTAL="0", CARGO_PROFILE_DEV_DEBUG="0", CARGO_PROFILE_TEST_DEBUG="0")
    result.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull, GIT_TERMINAL_PROMPT="0")
    result.update(MARTENSITE_CI_MODE="pr", GITHUB_EVENT_NAME="workflow_dispatch")
    return result


class InventoryCommandCapture:
    """Owns synchronous command receipts; instantiate with an explicit credential-free environment."""
    def __init__(self, product: Path, output: Path, environment: Mapping[str, str]) -> None:
        self.product = product
        self.output = output
        target = product.parent / "martensite-native-compiler-build"
        self.environment = inventory_capture_environment(environment, target)
        self.records: list[dict[str, object]] = []

    def run(self, arguments: list[str]) -> subprocess.CompletedProcess[bytes]:
        """Retain the actual bytes and status; e.g. run(['git', 'rev-parse', 'HEAD'])."""
        prefix = f"command-{len(self.records):03d}"
        stdout, stderr = (self.output / (prefix + suffix) for suffix in (".stdout.log", ".stderr.log"))
        with stdout.open("wb") as output, stderr.open("wb") as errors:
            result = subprocess.run(arguments, cwd=self.product, env=self.environment,
                                    stdout=output, stderr=errors, check=False)
        self.records.append(dict(argv=arguments, returncode=result.returncode, prefix=prefix))
        return subprocess.CompletedProcess(arguments, result.returncode, stdout.read_bytes(), stderr.read_bytes())

    def checked(self, arguments: list[str]) -> bytes:
        """Reject failed setup while retaining its receipt; e.g. checked(['git', 'status'])."""
        result = self.run(arguments)
        if result.returncode:
            raise RuntimeError(f"{arguments!r} returned {result.returncode}; expected zero")
        return result.stdout


def checkout_inventory_product(commands: InventoryCommandCapture) -> None:
    """Pin public source and a genuine immutable manual-PR base; call on a fresh directory."""
    commands.checked(["git", "init"])
    commands.checked(["git", "remote", "add", "origin", "https://github.com/Oblikovati/Martensite.git"])
    commands.checked(["git", "fetch", "--depth=2", "origin", PRODUCT_SHA])
    commands.checked(["git", "checkout", "--detach", PRODUCT_SHA])
    # Manual PR mode reads origin/main; this is the selected historical comparison ref.
    commands.checked(["git", "update-ref", "refs/remotes/origin/main", BASE_SHA])
    actual = commands.checked(["git", "merge-base", "HEAD", "origin/main"]).decode().strip()
    if actual != BASE_SHA:
        raise ValueError(f"comparison base {actual!r}; expected ancestor {BASE_SHA}")
    commands.checked(["git", "submodule", "update", "--init", "--depth=1"])


def inventory_tracked_seal(commands: InventoryCommandCapture, entry: bytes) -> tuple[str, str]:
    """Hash actual tracked bytes, preserving gitlinks; use for one ls-files --stage entry."""
    description, raw_name = entry.decode().split("\t", 1)
    mode, blob, stage = description.split()
    path = commands.product / raw_name
    if stage != "0":
        raise ValueError(f"index stage {stage!r} for {raw_name}; expected zero")
    if mode == "160000":
        actual = commands.checked(["git", "-C", str(path), "rev-parse", "HEAD"]).decode().strip()
        dirty = commands.checked(["git", "-C", str(path), "status", "--porcelain"])
        if actual != blob or dirty:
            raise ValueError(f"gitlink {raw_name}: {actual}/{dirty!r}; expected clean {blob}")
        return raw_name, "gitlink:" + blob
    payload = os.readlink(path).encode() if mode == "120000" else path.read_bytes()
    return raw_name, hashlib.sha256(payload).hexdigest()


def inventory_source_bookend(commands: InventoryCommandCapture) -> dict[str, object]:
    """Seal clean checked-out inputs; call before and after the unchanged product command."""
    head = commands.checked(["git", "rev-parse", "HEAD"]).decode().strip()
    status = commands.checked(["git", "status", "--porcelain=v1", "--untracked-files=all"])
    if head != PRODUCT_SHA or status:
        raise ValueError(f"source {head}/{status!r}; expected clean {PRODUCT_SHA}")
    entries = commands.checked(["git", "ls-files", "--stage", "-z"]).split(b"\0")
    hashes = dict(inventory_tracked_seal(commands, entry) for entry in entries if entry)
    return dict(revision=head, status=status.decode(), tracked_sha256=hashes)


def inventory_native_target(commands: InventoryCommandCapture, machine: str) -> str:
    """Require a matching native compiler; e.g. machine='arm64' selects native ARM evidence."""
    triple = NATIVE_TARGETS.get(machine)
    compiler = commands.checked(["rustc", "+" + TOOLCHAIN, "-vV"]).decode()
    pinned = tomllib.loads((commands.product / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
    if triple is None or pinned != TOOLCHAIN or not compiler.startswith("rustc " + TOOLCHAIN + " "):
        raise ValueError(f"machine/toolchain {machine}/{pinned}/{compiler!r}; expected pinned native Mac")
    if f"host: {triple}\n" not in compiler:
        raise ValueError(f"compiler {compiler!r}; expected native host {triple}")
    return triple


def inventory_checks_command(triple: str) -> list[str]:
    """Select the supported no-application-tests path; e.g. inventory_checks_command(native)."""
    command = ["cargo", "+" + TOOLCHAIN, "xtask", "ci", "run"]
    command.extend(["--target", triple, "--checks-only"])
    return command


def retain_inventory_reports(product: Path, output: Path) -> dict[str, object]:
    """Retain reports even on comparator failure; call from the capture's finalization path."""
    retained: dict[str, str] = {}
    destination = output / "inventory"
    destination.mkdir()
    for name in INVENTORY_NAMES:
        source = product / "target" / "unsafe-inventory" / name
        if source.is_file():
            payload = source.read_bytes()
            (destination / name).write_bytes(payload)
            retained[name] = hashlib.sha256(payload).hexdigest()
    return dict(files=retained, all_three_present=len(retained) == len(INVENTORY_NAMES))


def inventory_comparison_observation(product: Path, report: dict[str, object]) -> dict[str, object]:
    """Compare retained fields without approval; actual gate status remains separately recorded."""
    target = report.get("target")
    if target not in NATIVE_TARGETS.values():
        raise ValueError(f"inventory target {target!r}; expected a supported native Mac triple")
    baseline = json.loads((product / "docs/features/safe-toolkits/compiler-baselines" / (str(target) + ".json")).read_text())
    provenance, diagnostics = report.get("provenance"), report.get("diagnostics")
    if not isinstance(provenance, dict) or not isinstance(diagnostics, dict):
        raise ValueError("inventory provenance/diagnostics; expected JSON objects")
    compiler = str(provenance.get("compiler", "")).splitlines()
    matching = dict(target=baseline["target"] == target,
                    compiler=bool(compiler) and baseline["compiler"] == compiler[0],
                    summary=baseline["summary"] == diagnostics.get("workspace_summary"))
    return dict(verdict="match" if all(matching.values()) else "mismatch", field_matches=matching)


def inventory_observations(output: Path, product: Path | None = None) -> dict[str, object]:
    """Describe compiler output without admitting expectations; use after raw file retention."""
    report = output / "inventory" / "latest.json"
    stream = output / "inventory" / "compiler.stdout.jsonl"
    if not report.is_file() or not stream.is_file():
        return dict(report_present=False, compiler_success=None)
    parsed = json.loads(report.read_text())
    events = [json.loads(line) for line in stream.read_text().splitlines() if line.strip()]
    finished = [event for event in events if event.get("reason") == "build-finished"]
    comparison = inventory_comparison_observation(product, parsed) if product else None
    return dict(report_present=True, target=parsed.get("target"), provenance=parsed.get("provenance"),
                compiler_success=finished[-1].get("success") if finished else None,
                observed_baseline=comparison)


def execute_inventory_capture(commands: InventoryCommandCapture, receipt: dict[str, object], machine: str) -> int:
    """Run authentic checks and preserve its status; applications are disabled by checks-only."""
    checkout_inventory_product(commands)
    receipt["before"] = inventory_source_bookend(commands)
    target = inventory_native_target(commands, machine)
    receipt["native_target"] = target
    stale = commands.product / "target" / "unsafe-inventory"
    if any((stale / name).exists() for name in INVENTORY_NAMES):
        raise ValueError(f"inventory directory {stale}; expected no previous reports")
    receipt["command"] = inventory_checks_command(target)
    (commands.output / "before-command.json").write_text(json.dumps(receipt, indent=2) + "\n")
    result = commands.run(inventory_checks_command(target))
    receipt["command_returncode"] = result.returncode
    receipt["after"] = inventory_source_bookend(commands)
    if receipt["before"] != receipt["after"]:
        raise ValueError("source bookends differ; expected unchanged compiler inputs")
    return result.returncode


def finish_inventory_capture(commands: InventoryCommandCapture, receipt: dict[str, object], status: int) -> int:
    """Seal actual results even on malformed reports; this never admits an expected baseline."""
    try:
        receipt["reports"] = retain_inventory_reports(commands.product, commands.output)
        receipt["observations"] = inventory_observations(commands.output, commands.product)
    except (OSError, ValueError, TypeError, AttributeError) as error:
        receipt["retention_error"] = str(error)
    receipt["commands"] = commands.records
    receipt["transport_returncode"] = status if status else 1
    receipt["transport_exit_policy"] = "No baseline admission: preserve nonzero actual gate status; an unexpected zero remains observed but unaccepted with transport exit one."
    (commands.output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    return status if status else 1


def capture_inventory_evidence(
    product: Path, output: Path, environment: Mapping[str, str], machine: str,
    process_type: type[InventoryCommandCapture] = InventoryCommandCapture,
) -> int:
    """Keep unreviewed evidence outside source; provide fresh product/output paths."""
    if output.is_relative_to(product):
        raise ValueError(f"output {output}; expected outside product {product}")
    product.mkdir(parents=True, exist_ok=False)
    output.mkdir(parents=True, exist_ok=False)
    commands = process_type(product, output, environment)
    receipt: dict[str, object] = dict(product=PRODUCT_SHA, base=BASE_SHA, machine=machine,
                                    platform=platform.platform(), expectations_accepted=False, human_review_status="pending")
    receipt["environment_keys"] = sorted(commands.environment)
    receipt["transport"] = {key: environment.get(key) for key in ("GITHUB_SHA", "GITHUB_RUN_ID", "GITHUB_RUN_ATTEMPT", "GITHUB_JOB", "GITHUB_REPOSITORY", "RUNNER_OS", "RUNNER_ARCH")}
    status = 1
    try:
        status = execute_inventory_capture(commands, receipt, machine)
    except (OSError, ValueError, RuntimeError) as error:
        receipt["capture_error"] = str(error)
    return finish_inventory_capture(commands, receipt, status)


def main() -> int:
    """Capture only on a real Mac runner; run this script from the safe transport checkout."""
    if platform.system() != "Darwin":
        raise RuntimeError(f"host {platform.system()}; expected native Darwin")
    temporary = Path(os.environ["RUNNER_TEMP"])
    product = temporary / "martensite-native-compiler-product"
    environment = dict(os.environ)
    output = Path.cwd() / "target" / "macos-cpu-pipe" / "compiler-inventory"
    return capture_inventory_evidence(product, output, environment, platform.machine())


if __name__ == "__main__":
    sys.exit(main())
