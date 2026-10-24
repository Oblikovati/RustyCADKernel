"""Require the unchanged immutable-stage assertion red before correction and green after (#265)."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess

PRODUCT = Path("product").resolve()
OUTPUT = Path("target/macos-immutable-stage").resolve()
REVISION = "ab4bad4fce774001b192fd20ac4d7a9010e68fea"
SOURCE = "crates/martensite-tkernel/src/osd/document_file/macos/tests.rs"
CASE = "osd::document_file::backend::tests::denied_stage_policy_keeps_original_document_unchanged"
ORIGINAL_SHA256 = "0006aeca6e4886d30a9187676c0b6689397e310559c93ff4fa735a82beba5b3c"
FIXED_SHA256 = "a1d46a1f4ee9b0994c8574e8deb7eb6a80937088a247bd1342d8a604135c9b9b"
OLD = b'Command::new("/bin/chflags")'
NEW = b'Command::new("/usr/bin/chflags")'


def git_text(*arguments: str) -> str:
    """Read immutable source state; `git_text('rev-parse', 'HEAD')`."""
    output = subprocess.check_output(["git", *arguments], cwd=PRODUCT, text=True)
    return output.strip()


def run_focused(label: str) -> subprocess.CompletedProcess[str]:
    """Execute only the unchanged exact native unit case; `run_focused('original')`."""
    environment = focused_environment()
    arguments = focused_arguments()
    result = subprocess.run(
        arguments, cwd=PRODUCT, env=environment, capture_output=True, text=True
    )
    (OUTPUT / f"{label}.stdout.log").write_text(result.stdout)
    (OUTPUT / f"{label}.stderr.log").write_text(result.stderr)
    return result


def focused_environment() -> dict[str, str]:
    """Keep both executions on one bounded native build configuration; `focused_environment()`."""
    environment = dict(os.environ, CARGO_TERM_COLOR="never", CARGO_BUILD_JOBS="2")
    environment.update(CARGO_PROFILE_DEV_DEBUG="0", CARGO_PROFILE_TEST_DEBUG="0")
    environment.update(
        CARGO_INCREMENTAL="0",
        CARGO_TARGET_DIR=str(Path("target/macos265-cargo").resolve()),
    )
    return environment


def focused_arguments() -> list[str]:
    """Require a single fully qualified test rather than a filtered approximation; `focused_arguments()`."""
    arguments = [
        "cargo",
        "+1.99.0",
        "test",
        "--locked",
        "-p",
        "martensite-tkernel",
        "--lib",
    ]
    return arguments + [CASE, "--", "--exact", "--nocapture", "--test-threads=1"]


def require_focused_result(
    result: subprocess.CompletedProcess[str], passes: bool
) -> None:
    """Reject a compile failure or unrelated/ignored case; `require_focused_result(result, True)`."""
    text = result.stdout + result.stderr
    expected = "ok" if passes else "FAILED"
    assert re.search(rf"^test {re.escape(CASE)} \.\.\. {expected}$", text, re.M), text
    summary = (
        "1 passed; 0 failed; 0 ignored" if passes else "0 passed; 1 failed; 0 ignored"
    )
    assert summary in text, text
    assert result.returncode == (0 if passes else 101), result.returncode
    if not passes:
        assert 'kind: NotFound, message: "No such file or directory"' in text, text


def require_product_source() -> bytes:
    """Pin the real native product baseline and unchanged checkout; `require_product_source()`."""
    assert platform.system() == "Darwin", platform.platform()
    assert git_text("rev-parse", "HEAD") == REVISION
    assert not git_text("status", "--porcelain=v1", "--untracked-files=all")
    original = (PRODUCT / SOURCE).read_bytes()
    assert original.count(OLD) == 1 and NEW not in original
    assert hashlib.sha256(original).hexdigest() == ORIGINAL_SHA256
    return original


def red_then_green(original: bytes) -> dict[str, object]:
    """Change only the fixture utility path and restore source after both executions; `red_then_green(original)`."""
    corrected = original.replace(OLD, NEW)
    try:
        red = run_focused("original")
        require_focused_result(red, False)
        (PRODUCT / SOURCE).write_bytes(corrected)
        green = run_focused("corrected")
        require_focused_result(green, True)
        return comparison_receipt(original, corrected, red.returncode, green.returncode)

    finally:
        (PRODUCT / SOURCE).write_bytes(original)


def comparison_receipt(
    original: bytes, corrected: bytes, red: int, green: int
) -> dict[str, object]:
    """Bind the successful source to the actual one-line product fix; `comparison_receipt(old, new, 101, 0)`."""
    corrected_hash = hashlib.sha256(corrected).hexdigest()
    assert corrected_hash == FIXED_SHA256, corrected_hash
    return {
        "original_returncode": red,
        "corrected_returncode": green,
        "original_source_sha256": hashlib.sha256(original).hexdigest(),
        "corrected_source_sha256": corrected_hash,
        "only_helper_executable_path_changed": corrected.replace(NEW, OLD) == original,
    }


def main() -> int:
    """Retain focused native proof without accepting skipped cases; `main()`."""
    receipt: dict[str, object] = {"accepted": False, "baseline": REVISION, "case": CASE}
    try:
        receipt.update(red_then_green(require_product_source()))
        assert not git_text("status", "--porcelain=v1", "--untracked-files=all")
        receipt.update(accepted=True, machine=platform.machine(), restored_clean=True)
    except (AssertionError, OSError, subprocess.CalledProcessError) as error:
        receipt["error"] = str(error)
    (OUTPUT / "focused-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt))
    return 0 if receipt["accepted"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
