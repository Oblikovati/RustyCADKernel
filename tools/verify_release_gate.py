#!/usr/bin/env python3
"""Execute the shared local/CI gate; e.g. python3 tools/verify_release_gate.py --gate tools."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import shlex
import subprocess
from typing import Protocol

from release_verification_plan import (
    GATE_NAMES,
    GateCommand,
    VerificationGate,
    verification_gates,
)


class GateExecutor(Protocol):
    """Named execution boundary allows tests to observe commands without heavy work."""

    def run(self, command: GateCommand, root: Path, environment: dict[str, str]) -> int:
        """Execute exactly one command; e.g. executor.run(command, root, environment)."""
        ...


class NativeGateExecutor:
    """Preserve child streams/status and the workflow's performance log location."""

    def run(self, command: GateCommand, root: Path, environment: dict[str, str]) -> int:
        """Execute argv without shell parsing; e.g. executor.run(command, root, env)."""
        if command.stdout_path is None:
            return subprocess.run(command.argv, cwd=root, env=environment).returncode
        output = root / command.stdout_path
        output.parent.mkdir(parents=True, exist_ok=True)
        with output.open("w") as stream:
            return subprocess.run(
                command.argv, cwd=root, env=environment, stdout=stream
            ).returncode


def run_gate(
    gate: VerificationGate,
    root: Path,
    inherited: dict[str, str],
    executor: GateExecutor,
) -> int:
    """Stop at the original failing command; e.g. run_gate(gate, root, env, executor)."""
    for command in gate.commands:
        print("+ " + shlex.join(command.argv), flush=True)
        environment = dict(inherited)
        environment.update(command.env)
        status = executor.run(command, root, environment)
        if status:
            # Shell PIPESTATUS convention preserves a child killed by a signal.
            return status if status > 0 else 128 - status
    return 0


def gate_arguments(argv: list[str] | None) -> argparse.Namespace:
    """Require a named gate; e.g. gate_arguments(['--gate', 'inventory'])."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--gate", required=True, choices=GATE_NAMES)
    parser.add_argument(
        "--root", type=Path, default=Path(__file__).resolve().parents[1]
    )
    parser.add_argument("--source", type=Path)
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    """Run one shared plan entry; e.g. main(['--gate', 'rust-prechecks'])."""
    args = gate_arguments(argv)
    root = args.root.resolve()
    source = args.source if args.source is not None else root / "target/occt-source"
    gates = verification_gates(root, source)
    gate = next(gate for gate in gates if gate.name == args.gate)
    return run_gate(gate, root, dict(os.environ), NativeGateExecutor())


if __name__ == "__main__":
    raise SystemExit(main())
