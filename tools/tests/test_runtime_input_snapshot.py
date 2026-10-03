"""Bind execution evidence to the input bytes that existed before the run."""

from __future__ import annotations

import pathlib
import unittest
from unittest.mock import patch

from legacy_runtime_evidence import (
    require_unchanged_runtime_inputs,
    runtime_input_hashes,
)


class FakeRuntimeInputFiles:
    """Supply deterministic file names and digests without filesystem I/O."""

    def __init__(self) -> None:
        self.root = pathlib.Path("/fixture")
        self.hashes = {"Cargo.toml": "a" * 64, "crates/kernel.rs": "b" * 64}

    def paths(self, root: pathlib.Path) -> set[str]:
        if root != self.root:
            raise ValueError(f"root={root}; expected {self.root}")
        return set(self.hashes)

    def digest(self, path: pathlib.Path) -> str:
        return self.hashes[path.relative_to(self.root).as_posix()]


class RuntimeInputSnapshotTests(unittest.TestCase):
    def test_hashes_capture_each_discovered_input(self) -> None:
        fixture = FakeRuntimeInputFiles()
        with patch(
            "legacy_runtime_evidence.tracked_runtime_inputs", fixture.paths
        ), patch("legacy_runtime_evidence.file_digest", fixture.digest):
            self.assertEqual(runtime_input_hashes(fixture.root), fixture.hashes)

    def test_unchanged_inputs_are_accepted(self) -> None:
        require_unchanged_runtime_inputs({"a": "digest"}, {"a": "digest"})

    def test_changed_added_or_removed_input_rejects_evidence(self) -> None:
        before = {"a": "digest"}
        for after in ({"a": "edited"}, {}, {"a": "digest", "b": "added"}):
            with self.subTest(after=after), self.assertRaisesRegex(
                ValueError, "expected identical pre/post hashes"
            ):
                require_unchanged_runtime_inputs(before, after)
