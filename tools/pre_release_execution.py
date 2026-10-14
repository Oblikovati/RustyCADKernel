"""Record the exact shared workflow commands before a Martensite release tag."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import platform
import shlex
import shutil
import subprocess
from collections.abc import Callable, Mapping
from typing import BinaryIO

from release_inventory import Commands
from release_verification_plan import (
    VERIFICATION_ENV,
    GateCommand,
    VerificationGate,
    verification_gates,
)
from with_vulkan_validation import NativeValidationCacheIO, command_environment

PINNED_CARGO_TOOLS = (("nextest", "0.9.148"), ("llvm-cov", "0.9.1"), ("deny", "0.20.2"))


def local_gate_environment(root: Path, inherited: Mapping[str, str]) -> dict[str, str]:
    """Use CI's bounded artifact and driver policy; example: local_gate_environment(root, os.environ)."""
    selected = native_gate_environment(root, bounded_ci_environment(inherited))
    selected["VK_DRIVER_FILES"] = str(require_lavapipe_driver())
    return command_environment(
        root / "target/vulkan-validation",
        selected,
        NativeValidationCacheIO(),
        platform.system(),
        platform.machine(),
    )


def native_gate_environment(root: Path, inherited: Mapping[str, str]) -> dict[str, str]:
    """Allow an unprivileged distribution tool cache; example: native_gate_environment(root, env)."""
    selected = dict(inherited)
    cache = root / "target/native-ci-tools/usr/bin"
    if cache.is_dir():
        selected["PATH"] = str(cache) + os.pathsep + selected.get("PATH", os.defpath)
    require_native_gate_tools(selected, shutil.which)
    return selected


def require_native_gate_tools(
    environment: Mapping[str, str], lookup: Callable[[str, int, str | None], str | None]
) -> None:
    """Reject missing CI prerequisites before compilation; example: require_native_gate_tools(env, shutil.which)."""
    tools = ("xvfb-run", "Xvfb", "xauth", "spirv-val", "glslangValidator", "pkg-config")
    missing = [
        tool
        for tool in tools
        if lookup(tool, os.F_OK | os.X_OK, environment.get("PATH")) is None
    ]
    if missing:
        raise ValueError(
            f"native CI tools={missing!r}: expected executable workflow prerequisites in PATH"
        )


def bounded_ci_environment(inherited: Mapping[str, str]) -> dict[str, str]:
    """Keep CI's profile pool and graphics sampling; example: bounded_ci_environment(environment)."""
    selected = dict(inherited)
    selected.update(VERIFICATION_ENV)
    selected["CARGO_BUILD_JOBS"] = "2"
    remove_verification_overrides(selected)
    return selected


def remove_verification_overrides(selected: dict[str, str]) -> None:
    """Keep committed aliases and toolchain authoritative; example: remove_verification_overrides(env)."""
    fixed = (
        "CARGO_TARGET_DIR",
        "CARGO_LLVM_COV_TARGET_DIR",
        "VK_ICD_FILENAMES",
        "LLVM_TARGET_FEATURES",
        "RUSTUP_TOOLCHAIN",
        "CARGO_BUILD_TARGET",
        "RUSTFLAGS",
        "RUSTDOCFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
    )
    for variable in tuple(selected):
        if variable in fixed or variable.startswith("CARGO_ALIAS_"):
            selected.pop(variable)


def require_lavapipe_driver() -> Path:
    """Require the same software backend as CI; example: require_lavapipe_driver().name."""
    drivers = sorted(Path("/usr/share/vulkan/icd.d").glob("lvp_icd*.json"))
    if not drivers:
        raise ValueError("lavapipe ICD=[]: expected installed lvp_icd*.json")
    return drivers[0]


def require_cargo_tool_versions(root: Path, environment: Mapping[str, str]) -> None:
    """Reject different verification tools; example: require_cargo_tool_versions(root, environment)."""
    for tool, version in PINNED_CARGO_TOOLS:
        output = subprocess.check_output(
            ["cargo", tool, "--version"], cwd=root, env=dict(environment), text=True
        ).strip()
        if version not in output.split():
            raise ValueError(
                f"cargo {tool} version={output!r}: expected {version!r}, install cargo-{tool} --version {version} --locked"
            )


def command_acceptance(
    command: GateCommand, log: Path, exit_code: int, evidence: Path
) -> dict[str, object]:
    """Bind actual execution to its bytes; example: command_acceptance(command, log, 0, evidence)."""
    return {
        "argv": list(command.argv),
        "env": dict(command.env),
        "stdout_path": str(command.stdout_path) if command.stdout_path else None,
        "exit_code": exit_code,
        "log": str(log.relative_to(evidence)),
        "sha256": hashlib.sha256(log.read_bytes()).hexdigest(),
    }


def execute_recorded_command(
    command: GateCommand, root: Path, environment: Mapping[str, str], log: Path
) -> int:
    """Preserve the original process exit without a shell; example: execute_recorded_command(command, root, env, log)."""
    selected = dict(environment)
    selected.update(command.env)
    print("+ " + shlex.join(command.argv), flush=True)
    with log.open("wb") as stderr:
        if command.stdout_path is None:
            return merged_command_stream(command, root, selected, stderr)
        output = root / command.stdout_path
        output.parent.mkdir(parents=True, exist_ok=True)
        with output.open("wb") as stdout:
            return subprocess.run(
                command.argv, cwd=root, env=selected, stdout=stdout, stderr=stderr
            ).returncode


def merged_command_stream(
    command: GateCommand, root: Path, environment: Mapping[str, str], stream: BinaryIO
) -> int:
    """Retain a failed child's original diagnostics; example: merged_command_stream(command, root, env, stream)."""
    argv = command.argv
    return subprocess.run(
        argv,
        cwd=root,
        env=dict(environment),
        stdout=stream,
        stderr=subprocess.STDOUT,
    ).returncode


def preserve_command_output(command: GateCommand, root: Path, evidence: Path) -> None:
    """Keep measured output outside the removable checkout; example: preserve_command_output(command, evidence)."""
    if command.stdout_path is None:
        return
    destination = evidence / "outputs" / command.stdout_path.name
    destination.parent.mkdir(exist_ok=True)
    shutil.copyfile(root / command.stdout_path, destination)


def gate_execution_environment(
    gate: VerificationGate, root: Path, environment: Mapping[str, str]
) -> dict[str, str]:
    """Isolate driver caches like separate CI jobs; example: gate_execution_environment(gate, root, env)."""
    group = "vulkan" if gate.name.startswith("vulkan-") else gate.name.split("-")[0]
    selected = dict(environment)
    selected["MESA_SHADER_CACHE_DIR"] = str(
        root / "target/workflow-driver-cache" / group
    )
    return selected


def verify_recorded_gate(
    gate: VerificationGate, root: Path, environment: Mapping[str, str], evidence: Path
) -> dict[str, object]:
    """Fail on the first real gate error; example: verify_recorded_gate(gate, root, env, evidence)."""
    if not gate.commands:
        raise ValueError(
            f"gate {gate.name!r} commands=(): expected at least one verification command"
        )
    selected = gate_execution_environment(gate, root, environment)
    commands: list[dict[str, object]] = []
    for index, command in enumerate(gate.commands):
        log = evidence / f"{gate.name}-{index:03}.log"
        exit_code = execute_recorded_command(command, root, selected, log)
        commands.append(command_acceptance(command, log, exit_code, evidence))
        result = {"name": gate.name, "exit_code": exit_code, "commands": commands}
        (evidence / f"{gate.name}.json").write_text(json.dumps(result, indent=2) + "\n")
        preserve_command_output(command, root, evidence)
        if exit_code:
            raise subprocess.CalledProcessError(exit_code, command.argv)
    return {"name": gate.name, "exit_code": 0, "commands": commands}


def verify_workflow_gates(
    root: Path, source: Path, environment: Mapping[str, str], evidence: Path
) -> list[dict[str, object]]:
    """Run the same registry consumed by CI; example: verify_workflow_gates(root, source, env, evidence)."""
    results: list[dict[str, object]] = []
    for gate in verification_gates(root, source):
        results.append(verify_recorded_gate(gate, root, environment, evidence))
    preserve_required_reports(root, evidence)
    return results


def preserve_required_reports(root: Path, evidence: Path) -> None:
    """Keep workflow uploads after deleting the checkout; example: preserve_required_reports(root, evidence)."""
    reports = (Path("target/lcov.info"), actual_junit_report(root).relative_to(root))
    destination = evidence / "outputs"
    destination.mkdir(exist_ok=True)
    for report in reports:
        source = root / report
        if not source.is_file() or not source.stat().st_size:
            raise ValueError(
                f"workflow report={str(source)!r}: expected nonempty generated report"
            )
        shutil.copyfile(source, destination / report.name)


def preserve_failure_diagnostics(root: Path, evidence: Path) -> None:
    """Keep partial reports and bounded package failure receipts; example: preserve_failure_diagnostics(root, evidence)."""
    reports = (
        "target/lcov.info",
        "target/nextest/ci/junit.xml",
        "target/llvm-cov-target/nextest/ci/junit.xml",
    )
    destination = evidence / "outputs"
    destination.mkdir(exist_ok=True)
    for relative in reports:
        source = root / relative
        if source.is_file():
            shutil.copyfile(source, destination / source.name)
    diagnostics = root / "target/release-diagnostics"
    if diagnostics.is_dir():
        shutil.copytree(
            diagnostics, evidence / "package-diagnostics", dirs_exist_ok=True
        )


def actual_junit_report(root: Path) -> Path:
    """Bind the generated nextest store, including coverage target overrides; example: actual_junit_report(root)."""
    stores = (
        "target/nextest/ci/junit.xml",
        "target/llvm-cov-target/nextest/ci/junit.xml",
    )
    present = [root / name for name in stores if (root / name).is_file()]
    if len(present) != 1:
        raise ValueError(
            f"nextest reports={present!r}: expected one generated CI JUnit report"
        )
    return present[0]


class CandidateEnvironmentCommands(Commands):
    """Inject the checked test environment into package verification."""

    def __init__(self, environment: Mapping[str, str], evidence: Path) -> None:
        self.environment = dict(environment)
        self.evidence = evidence
        self.evidence.mkdir(parents=True, exist_ok=True)
        self.index = 0
        self.last_stderr = ""

    def run(self, arguments: list[str], root: Path) -> str:
        """Retain package diagnostics and original failures; example: run(['cargo','test'], root)."""
        result = subprocess.run(
            arguments, cwd=root, env=self.environment, text=True, capture_output=True
        )
        self.last_stderr = result.stderr
        prefix = self.evidence / f"package-{self.index:03}"
        self.index += 1
        prefix.with_suffix(".stdout.log").write_text(result.stdout)
        prefix.with_suffix(".stderr.log").write_text(result.stderr)
        prefix.with_suffix(".json").write_text(
            json.dumps(
                {"argv": arguments, "root": str(root), "exit_code": result.returncode}
            )
            + "\n"
        )
        print(result.stderr, end="", flush=True)
        result.check_returncode()
        return result.stdout
