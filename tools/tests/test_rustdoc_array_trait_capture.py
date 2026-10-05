"""Verify trait paths using unmodified output from the pinned rustdoc compiler."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import unittest

from normalize_rustdoc_inventory import make_reachable_inventory

ROOT = Path(__file__).resolve().parents[2]
FIXTURE = ROOT / "tools/tests/fixtures/rustdoc-array-traits"


class RustdocArrayTraitCaptureTest(unittest.TestCase):
    def test_capture_is_source_bound_to_the_pinned_format(self) -> None:
        receipt = json.loads((FIXTURE / "capture.json").read_text())
        document = json.loads((FIXTURE / "rustdoc-61.json").read_text())
        self.assertEqual(document["format_version"], 61)
        for filename, field in (
            ("lib.rs", "source_sha256"),
            ("rustdoc-61.json", "rustdoc_sha256"),
        ):
            actual = hashlib.sha256((FIXTURE / filename).read_bytes()).hexdigest()
            self.assertEqual(actual, receipt[field])

    def test_array_and_slice_conversions_are_distinct_reachable_methods(self) -> None:
        document = json.loads((FIXTURE / "rustdoc-61.json").read_text())
        inventory = make_reachable_inventory(document, ROOT, "array_trait_fixture")
        paths = [entry["path"] for entry in inventory["declarations"]]
        for argument in ("&[u8; N]", "&[u8]", "&mut [u8; 4]"):
            path = f"array_trait_fixture::ByteCapture::<core::convert::From<{argument}>>::from"
            self.assertEqual(paths.count(path), 1, path)
