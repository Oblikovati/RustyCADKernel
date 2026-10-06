"""Toolkit scoping never rewrites source authority or invents compile configuration."""

from __future__ import annotations

import argparse
import copy
import unittest
from pathlib import Path

from api_inventory.capture_scope import owner_toolkits, scoped_commands, scoped_manifest
from api_inventory.inputs import HeaderJob, JSONObject, object_rows, select_header_jobs
from capture_occt_inventory import add_generated_header


def source_manifest() -> JSONObject:
    """Build multiply-owned and single-owner header fixtures for capture selection."""
    return {
        "revision": "pinned",
        "headers": [
            {"path": "base.hxx", "owners": [{"toolkit": "TKernel"}]},
            {"path": "geometry.hxx", "owners": [{"toolkit": "TKMath"}]},
            {
                "path": "shared.hxx",
                "owners": [{"toolkit": "TKMath"}, {"toolkit": "TKernel"}],
            },
            {"path": "base.lxx", "owners": [{"toolkit": "TKernel"}]},
        ],
    }


class CaptureScopeTests(unittest.TestCase):
    def test_all_scope_preserves_every_header(self) -> None:
        manifest = source_manifest()
        self.assertIs(scoped_manifest(manifest, []), manifest)
        self.assertEqual(
            owner_toolkits(object_rows(manifest["headers"], "headers")[2]),
            {"TKernel", "TKMath"},
        )

    def test_requested_scope_retains_fragment_and_shared_owners(self) -> None:
        manifest = source_manifest()
        original = copy.deepcopy(manifest)
        selected = scoped_manifest(manifest, ["TKernel", "TKernel"])
        self.assertEqual(
            [row["path"] for row in object_rows(selected["headers"], "headers")],
            ["base.hxx", "shared.hxx", "base.lxx"],
        )
        self.assertEqual(manifest, original)
        self.assertEqual(selected["revision"], "pinned")

    def test_unknown_scope_is_rejected(self) -> None:
        with self.assertRaisesRegex(
            ValueError, "requested toolkits.*Wrong.*expected manifest owners"
        ):
            scoped_manifest(source_manifest(), ["Wrong"])

    def test_shared_owner_uses_requested_real_command(self) -> None:
        commands = {
            "TKMath": ("geometry", ["-DGEOMETRY"]),
            "TKernel": ("foundation", ["-DFOUNDATION"]),
        }
        selected = scoped_manifest(source_manifest(), ["TKernel"])
        jobs, coverage = select_header_jobs(
            selected, scoped_commands(commands, ["TKernel"])
        )
        self.assertTrue(
            all(
                job.toolkit == "TKernel" and job.flags == ("-DFOUNDATION",)
                for job in jobs
            )
        )
        self.assertEqual(len(jobs), 2)
        self.assertEqual(coverage[2]["status"], "fragment_pending")
        self.assertEqual(scoped_commands(commands, []), commands)

    def test_generated_header_resolves_before_toolkit_cwd_changes(self) -> None:
        jobs = [HeaderJob("base.hxx", "TKernel", "/different/toolkit/build", ())]
        coverage: list[JSONObject] = []
        arguments = argparse.Namespace(
            database=Path("relative-build/compile_commands.json")
        )
        add_generated_header(jobs, coverage, arguments)
        self.assertTrue(Path(jobs[-1].input_path).is_absolute())
        self.assertEqual(
            jobs[-1].input_path,
            str(
                Path(
                    "relative-build/include/opencascade/Standard_Version.hxx"
                ).resolve()
            ),
        )
        self.assertEqual(coverage[-1]["path"], "@generated/Standard_Version.hxx")

    def test_missing_configuration_stays_in_scope(self) -> None:
        selected = scoped_manifest(source_manifest(), ["TKernel"])
        jobs, coverage = select_header_jobs(
            selected, scoped_commands({"TKMath": ("other", [])}, ["TKernel"])
        )
        self.assertFalse(jobs)
        self.assertTrue(
            all(row["status"] == "missing_compile_configuration" for row in coverage)
        )


if __name__ == "__main__":
    unittest.main()
