"""Capture real immutable-flag utility availability on native Mac runners (#265)."""

from __future__ import annotations

import json
import os
from pathlib import Path
import platform
import shutil
import stat
import subprocess
import tempfile

OUTPUT = Path("target/macos-immutable-stage")
CANDIDATES = ("/bin/chflags", "/usr/bin/chflags", "/usr/sbin/chflags", "/sbin/chflags")


def utility_inventory() -> dict[str, object]:
    """Record canonical candidates independently of PATH; `utility_inventory()`."""
    return {
        "path_lookup": shutil.which("chflags"),
        "candidates": {path: os.access(path, os.X_OK) for path in CANDIDATES},
    }


def change_flag(executable: str, fixture: Path, flag: str) -> dict[str, object]:
    """Record a real system command and native flag readback; `change_flag(tool, fixture, 'uchg')`."""
    arguments = [executable, flag, str(fixture)]
    result = subprocess.run(arguments, capture_output=True, text=True, check=False)
    return {
        "argv": arguments,
        "exit_code": result.returncode,
        "stdout": result.stdout,
        "stderr": result.stderr,
        "immutable": bool(fixture.stat().st_flags & stat.UF_IMMUTABLE),
    }


def exercise_immutable_flag(executable: str) -> list[dict[str, object]]:
    """Require real uchg and nouchg transitions with unchanged bytes; `exercise_immutable_flag(tool)`."""
    with tempfile.TemporaryDirectory(prefix="martensite-immutable-265-") as directory:
        fixture = Path(directory) / "replacement"
        fixture.write_bytes(b"replacement")
        return immutable_transitions(executable, fixture)


def immutable_transitions(executable: str, fixture: Path) -> list[dict[str, object]]:
    """Read back both real flag transitions and always clear the flag; `immutable_transitions(tool, fixture)`."""
    records = []
    try:
        for flag, expected in (("uchg", True), ("nouchg", False)):
            record = change_flag(executable, fixture, flag)
            records.append(record)
            assert record["exit_code"] == 0 and record["immutable"] == expected, record
            assert fixture.read_bytes() == b"replacement", str(fixture)
    finally:
        change_flag(executable, fixture, "nouchg")
    return records


def native_capture() -> dict[str, object]:
    """Refuse non-native execution and identify the actual utility; `native_capture()`."""
    assert platform.system() == "Darwin", platform.platform()
    inventory = utility_inventory()
    executables = [path for path in CANDIDATES if os.access(path, os.X_OK)]
    assert executables, inventory
    return {
        "system": platform.platform(),
        "machine": platform.machine(),
        "utility_inventory": inventory,
        "transitions": exercise_immutable_flag(executables[0]),
        "source_revision": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], text=True
        ).strip(),
    }


def main() -> int:
    """Preserve native observations on success and failure; `main()`."""
    OUTPUT.mkdir(parents=True, exist_ok=False)
    receipt: dict[str, object] = {"accepted": False}
    try:
        receipt.update(native_capture(), accepted=True)
    except (AssertionError, OSError) as error:
        receipt["error"] = str(error)
    (OUTPUT / "utility-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt))
    return 0 if receipt["accepted"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
