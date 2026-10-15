"""Approved native GTEST_SKIP cases stay pinned to their skip message."""

import unittest
import xml.etree.ElementTree as ET

from toolkit_native_skips import approved_native_skips, native_skip_observations


def skip_row(reason: str = "TODO OCC1: not done.") -> dict[str, str]:
    """An approved native skip ledger row; skip_row()."""
    return {"upstream_id": "Suite.Skips", "status": "upstream_skipped",
            "behavior_decision": "preserve_current", "native_skip_reason": reason}


def case_xml(name: str, message: str | None = None) -> ET.Element:
    """A native XML case, skipped with the message when one is given; case_xml('Skips', 'm')."""
    case = ET.Element("testcase", {"classname": "Suite", "name": name, "status": "run"})
    if message is not None:
        ET.SubElement(case, "skipped", {"message": "file.cxx:12\n" + message + "\n"})
    return case


class NativeSkipTests(unittest.TestCase):
    def test_approval_requires_reason_and_decision(self) -> None:
        for field in ("native_skip_reason", "behavior_decision"):
            row = skip_row()
            del row[field]
            with self.assertRaises(ValueError):
                approved_native_skips([row])
        self.assertEqual(approved_native_skips([skip_row(), {"status": "translated"}]),
                         {"Suite.Skips": "TODO OCC1: not done."})

    def test_approved_skip_is_split_from_running_cases(self) -> None:
        approved = approved_native_skips([skip_row()])
        cases = [case_xml("Runs"), case_xml("Skips", "TODO OCC1: not done.")]
        others, observed = native_skip_observations(cases, approved)
        self.assertEqual([case.attrib["name"] for case in others], ["Runs"])
        self.assertEqual(observed, [{"upstream_id": "Suite.Skips", "reason": "TODO OCC1: not done."}])

    def test_unapproved_or_changed_skip_fails(self) -> None:
        with self.assertRaises(ValueError):
            native_skip_observations([case_xml("Skips", "TODO OCC1: not done.")], {})
        with self.assertRaises(ValueError):
            native_skip_observations([case_xml("Skips", "another reason")],
                                     approved_native_skips([skip_row()]))

    def test_approval_lapses_when_the_case_runs_to_the_end(self) -> None:
        with self.assertRaises(ValueError):
            native_skip_observations([case_xml("Skips")], approved_native_skips([skip_row()]))


if __name__ == "__main__":
    unittest.main()
