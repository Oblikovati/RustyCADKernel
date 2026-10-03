"""Cargo's machine output determines the oracle binary, never a guessed path."""

import json
from pathlib import Path
import unittest

from api_inventory.inputs import JSONValue
from cargo_probe_artifact import cargo_message_rows, cargo_probe_executable


def executable_message(executable: JSONValue = "/custom/probe") -> str:
    return json.dumps(
        {
            "reason": "compiler-artifact",
            "target": {"name": "probe", "kind": ["example"]},
            "executable": executable,
        }
    )


def successful_build(*artifacts: str) -> str:
    finish = json.dumps({"reason": "build-finished", "success": True})
    return "\n".join([*artifacts, finish])


class CargoProbeArtifactTests(unittest.TestCase):
    def test_custom_path_is_selected_among_unrelated_messages(self) -> None:
        unrelated = json.dumps(
            {"reason": "compiler-artifact", "target": {"kind": ["lib"]}}
        )
        output = successful_build(unrelated, executable_message())
        self.assertEqual(cargo_probe_executable(output, "probe"), Path("/custom/probe"))

    def test_absent_or_duplicate_executables_fail(self) -> None:
        for artifacts in ((), (executable_message(), executable_message())):
            with self.subTest(artifacts=artifacts), self.assertRaisesRegex(
                ValueError, "expected one path"
            ):
                cargo_probe_executable(successful_build(*artifacts), "probe")

    def test_null_and_relative_executables_fail(self) -> None:
        for executable in (None, "relative/probe", 42):
            with self.subTest(executable=executable), self.assertRaises(ValueError):
                cargo_probe_executable(
                    successful_build(executable_message(executable)), "probe"
                )

    def test_unfinished_failed_and_duplicate_build_results_fail(self) -> None:
        for finish in (
            "",
            '{"reason":"build-finished","success":false}',
            '{"reason":"build-finished","success":1}',
            successful_build() + "\n" + successful_build(),
        ):
            with self.subTest(finish=finish), self.assertRaisesRegex(
                ValueError, "expected one successful build"
            ):
                cargo_probe_executable(executable_message() + "\n" + finish, "probe")

    def test_wrong_target_name_or_kind_fails(self) -> None:
        for message in (
            executable_message().replace('"probe"', '"other"'),
            executable_message().replace('"example"', '"bin"'),
        ):
            with self.subTest(message=message), self.assertRaisesRegex(
                ValueError, "expected one path"
            ):
                cargo_probe_executable(successful_build(message), "probe")

    def test_malformed_messages_fail_and_blank_lines_are_ignored(self) -> None:
        for message in ("not json", "[]", "null"):
            with self.subTest(message=message), self.assertRaises(ValueError):
                cargo_message_rows(message)
        self.assertEqual(cargo_message_rows("\n{}\n\n"), [{}])
