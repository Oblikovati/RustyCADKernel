#!/usr/bin/env python3
"""Verify the committed Martensite candidate before tagging; example: pre_release.py --tag v0.2.7 --source ../OCCT."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import subprocess
from typing import Protocol


class PreReleaseOperations(Protocol):
    """Own release effects; example: verify_then_tag(tag, FakePreReleaseOperations())."""

    def prepare(self, tag: str) -> Path:
        """Create the committed clean checkout; example: prepare('v0.2.7')."""
        ...

    def verify(self, checkout: Path) -> None:
        """Run every shared workflow gate; example: verify(checkout)."""
        ...

    def package(self, checkout: Path, tag: str) -> Path:
        """Verify untagged source packages; example: package(checkout, tag)."""
        ...

    def receipt(self, checkout: Path, tag: str, assets: Path) -> Path:
        """Bind successful checks and artifacts; example: receipt(checkout, tag, assets)."""
        ...

    def tag(self, checkout: Path, tag: str, receipt: Path) -> None:
        """Create the guarded immutable annotation; example: tag(checkout, tag, receipt)."""
        ...

    def dispose(self, checkout: Path) -> None:
        """Delete the temporary checkout, keeping evidence; example: dispose(checkout)."""
        ...


def verify_then_tag(tag: str, operations: PreReleaseOperations) -> Path:
    """Create no tag until all checks pass; example: verify_then_tag('v0.2.7', operations)."""
    checkout = operations.prepare(tag)
    try:
        operations.verify(checkout)
        assets = operations.package(checkout, tag)
        receipt = operations.receipt(checkout, tag, assets)
        operations.tag(checkout, tag, receipt)
        return receipt
    finally:
        # Failed verification must not leave another source/build checkout consuming disk.
        operations.dispose(checkout)


def parse_pre_release(argv: list[str] | None = None) -> argparse.Namespace:
    """Keep the candidate and oracle source explicit; example: parse_pre_release(['--tag','v0.2.7','--source','../OCCT'])."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument(
        "--root", type=Path, default=Path(__file__).resolve().parents[1]
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    """Run the mandatory local release entry point; example: main(['--tag','v0.2.7','--source','../OCCT'])."""
    from pre_release_operations import NativePreReleaseOperations

    options = parse_pre_release(argv)
    operations = NativePreReleaseOperations(
        options.root.resolve(), options.source.resolve(), dict(os.environ)
    )
    try:
        receipt = verify_then_tag(options.tag, operations)
    except subprocess.CalledProcessError as failure:
        print(
            f"Release verification failed: command={failure.cmd!r}, exit={failure.returncode}; expected every gate to pass."
        )
        return failure.returncode
    print(f"Verified and tagged {options.tag}; acceptance receipt: {receipt}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
