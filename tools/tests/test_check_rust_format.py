"""Regression coverage for Windows os error 206 in the v0.1.0 owner checks."""

import unittest
from check_rust_format import (
    ARGUMENT_LIMIT,
    command_units,
    format_batches,
    workspace_commands,
)


class RustFormatBatchTests(unittest.TestCase):
    def test_large_workspace_checks_every_root_once_with_bounded_commands(self) -> None:
        roots = [
            f"D:/long workspace/資源/crates/kernel/tests/case_{i:05d}.rs"
            for i in range(1800)
        ]
        commands = format_batches("2024", roots + roots[:10])
        self.assertGreater(len(commands), 1)
        self.assertEqual([root for cmd in commands for root in cmd[4:]], sorted(roots))
        self.assertTrue(all(command_units(cmd) <= ARGUMENT_LIMIT for cmd in commands))
        self.assertTrue(
            all(
                cmd[:4] == ["rustfmt", "--check", "--edition", "2024"]
                for cmd in commands
            )
        )

    def test_windows_quoting_and_non_bmp_characters_count_utf16_units(self) -> None:
        self.assertEqual(command_units(["x", "a b", "😀"]), 11)

    def test_empty_roots_do_not_launch_rustfmt_without_files(self) -> None:
        self.assertEqual(format_batches("2024", []), [])

    def test_oversized_single_root_reports_the_offending_path(self) -> None:
        root = "x" * ARGUMENT_LIMIT + ".rs"
        with self.assertRaisesRegex(
            ValueError, "expected command below 24000 UTF-16 units"
        ) as error:
            format_batches("2024", [root])
        self.assertIn(root, str(error.exception))

    def test_all_workspace_target_kinds_and_editions_are_checked(self) -> None:
        def target(path: str, edition: str) -> dict[str, str]:
            return {"src_path": path, "edition": edition}

        metadata = {
            "workspace_members": ["kernel"],
            "packages": [
                {"id": "external", "targets": [target("/external.rs", "2021")]},
                {
                    "id": "kernel",
                    "targets": [
                        target("/src/lib.rs", "2024"),
                        target("/tests/test.rs", "2024"),
                        target("/examples/view.rs", "2024"),
                        target("/build.rs", "2021"),
                        target("/native.c", "2024"),
                    ],
                },
            ],
        }
        commands = workspace_commands(metadata)
        self.assertEqual(
            commands,
            [
                ["rustfmt", "--check", "--edition", "2021", "/build.rs"],
                [
                    "rustfmt",
                    "--check",
                    "--edition",
                    "2024",
                    "/examples/view.rs",
                    "/src/lib.rs",
                    "/tests/test.rs",
                ],
            ],
        )
