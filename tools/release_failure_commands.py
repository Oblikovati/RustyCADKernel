"""Owned Commands decorator retaining cold-test failures beyond temporary cleanup."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import sys
from typing import Protocol, runtime_checkable

from release_failure_inputs import failure_inputs
from release_inventory import Commands

LOG_LIMIT = 16 * 1024 * 1024


@runtime_checkable
class StderrCommands(Protocol):
    """Optional captured stderr supplied by the real subprocess boundary."""

    last_stderr: str


class ReleaseDiagnosticCommands:
    """Record commands without retries/warmup; e.g. ReleaseDiagnosticCommands(runner, output)."""

    def __init__(self, commands: Commands, output: Path) -> None:
        self.commands = commands
        self.output = output
        self.sequence = 0
        self.output.mkdir(parents=True, exist_ok=False)

    def run(self, arguments: list[str], root: Path) -> str:
        """Preserve the original exception; e.g. run(['cargo', 'test'], stage)."""
        self.sequence += 1
        try:
            stdout = self.commands.run(arguments, root)
        except subprocess.CalledProcessError as failure:
            self.preserve_failure(arguments, root, failure)
            raise
        stderr = (
            self.commands.last_stderr
            if isinstance(self.commands, StderrCommands)
            else ""
        )
        self.record(arguments, root, stdout, stderr, 0)
        return stdout

    def preserve_failure(
        self, arguments: list[str], root: Path, failure: subprocess.CalledProcessError
    ) -> None:
        """Diagnostic I/O cannot replace the original red; e.g. preserve_failure(args, cwd, error)."""
        try:
            self.record(
                arguments,
                root,
                failure.output or "",
                failure.stderr or "",
                failure.returncode,
            )
            self.capture_failure(root, (failure.output or "") + (failure.stderr or ""))
        except Exception as diagnostic_failure:
            print(
                json.dumps({"release_diagnostic_error": str(diagnostic_failure)}),
                file=sys.stderr,
            )

    def record(
        self, arguments: list[str], root: Path, stdout: str, stderr: str, exit_code: int
    ) -> None:
        """Write bounded streams and exact command identity; e.g. record(args, cwd, out, err, 1)."""
        prefix = f"command-{self.sequence:04d}"
        streams = {
            name: self.record_stream(prefix, name, content)
            for name, content in (("stdout", stdout), ("stderr", stderr))
        }
        self.write_json(
            f"{prefix}.json",
            {
                "arguments": arguments,
                "cwd": str(root),
                "exit": exit_code,
                "streams": streams,
            },
        )

    def record_stream(self, prefix: str, name: str, content: str) -> dict[str, object]:
        """Limit uploaded stream bytes explicitly; e.g. record_stream('command-1', 'stderr', text)."""
        filename = f"{prefix}-{name}.log"
        encoded = content.encode("utf-8", errors="replace")
        (self.output / filename).write_bytes(encoded[:LOG_LIMIT])
        return {
            "file": filename,
            "bytes": len(encoded),
            "truncated": len(encoded) > LOG_LIMIT,
        }

    def capture_failure(self, stage: Path, transcript: str) -> None:
        """Collect metadata after the cold failure only; e.g. capture_failure(stage, stderr)."""
        try:
            self.write_json("failure-inputs.json", failure_inputs(stage, transcript))
        except Exception as diagnostic_failure:
            # Evidence collection must never mask the original failed release command.
            self.write_json(
                "capture-error.json",
                {"error": str(diagnostic_failure), "stage": str(stage)},
            )

    def write_json(self, filename: str, receipt: dict[str, object]) -> None:
        """Keep uploadable receipts separate from stage; e.g. write_json('failure.json', receipt)."""
        content = json.dumps(receipt, indent=2, sort_keys=True) + "\n"
        (self.output / filename).write_text(content, encoding="utf-8")
