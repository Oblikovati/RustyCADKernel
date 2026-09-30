"""Source-disabled originals remain failures, separately from compatibility regressions."""

from pathlib import Path
import unittest
from unittest import mock
import xml.etree.ElementTree as ET
import verify_toolkit
from toolkit_source_disabled import (
    source_disabled_rows,
    active_native_cases,
    forced_disabled_observations,
)
from verify_toolkit import check_native_cases, verify_forced_disabled


def disabled_mapping() -> dict[str, str]:
    """Explicit pinned decision fixture; disabled_mapping()."""
    return {
        "upstream_id": "Suite.DISABLED_Original",
        "status": "upstream_disabled",
        "source_sha256": "a" * 64,
        "behavior_decision": "preserve_current",
        "native_disabled_reason": "Inline entries are retained by OCCT.",
        "native_disabled_expected_failure": "actual: 8 vs 8",
        "rust_test": "compatibility_retains_entries",
        "rust_reference": "original_assertion",
    }


def disabled_xml(status: str = "notrun", outcome: str = "suppressed") -> ET.Element:
    """The original's default suppressed XML case; disabled_xml()."""
    attributes = {
        "classname": "Suite",
        "name": "DISABLED_Original",
        "status": status,
        "result": outcome,
    }
    return ET.Element("testcase", attributes)


def forced_xml(message: str = "expected fewer, actual: 8 vs 8") -> ET.Element:
    """The forced run executes and fails the original assertion; forced_xml()."""
    case = disabled_xml("run", "completed")
    ET.SubElement(case, "failure", {"message": message})
    return case


class SourceDisabledProvenanceTests(unittest.TestCase):
    def test_explicit_disabled_decision_requires_every_provenance_field(self) -> None:
        fields = [
            "source_sha256",
            "behavior_decision",
            "native_disabled_reason",
            "native_disabled_expected_failure",
            "rust_test",
            "rust_reference",
        ]
        for field in fields:
            row = disabled_mapping()
            del row[field]
            with self.subTest(field=field), self.assertRaises(ValueError):
                source_disabled_rows([row])

    def test_only_source_disabled_names_can_use_the_status(self) -> None:
        row = disabled_mapping()
        row["upstream_id"] = "Suite.Active"
        with self.assertRaisesRegex(ValueError, "DISABLED_"):
            source_disabled_rows([row])

    def test_skip_and_translated_disabled_rows_are_not_sanctioned(self) -> None:
        for status in ["upstream_skipped", "translated", "unknown"]:
            row = disabled_mapping()
            row["status"] = status
            with self.subTest(status=status), self.assertRaises(ValueError):
                source_disabled_rows([row])

    def test_compatibility_cannot_be_the_original_assertion(self) -> None:
        row = disabled_mapping()
        row["rust_test"] = row["rust_reference"]
        with self.assertRaisesRegex(ValueError, "separate"):
            source_disabled_rows([row])

    def test_hash_and_duplicate_mapping_are_rejected(self) -> None:
        row = disabled_mapping()
        with self.assertRaisesRegex(ValueError, "one disabled"):
            source_disabled_rows([row, row])
        row["source_sha256"] = "unpinned"
        with self.assertRaisesRegex(ValueError, "SHA256"):
            source_disabled_rows([row])


class SourceDisabledDefaultRunTests(unittest.TestCase):
    def test_suppressed_original_is_not_a_native_pass(self) -> None:
        disabled = source_disabled_rows([disabled_mapping()])
        active = active_native_cases([disabled_xml()], disabled)
        self.assertEqual(active, [])
        self.assertEqual(check_native_cases(active, {}), [])

    def test_missing_duplicate_or_executed_disabled_case_is_rejected(self) -> None:
        disabled = source_disabled_rows([disabled_mapping()])
        for cases in [[], [disabled_xml(), disabled_xml()], [forced_xml()]]:
            with self.subTest(count=len(cases)), self.assertRaises(ValueError):
                active_native_cases(cases, disabled)

    def test_unsanctioned_disabled_and_skips_still_fail(self) -> None:
        with self.assertRaisesRegex(ValueError, "Skipped"):
            check_native_cases(active_native_cases([disabled_xml()], {}), {})
        case = disabled_xml("run", "completed")
        ET.SubElement(case, "skipped")
        with self.assertRaises(ValueError):
            active_native_cases([case], source_disabled_rows([disabled_mapping()]))

    def test_active_regression_is_never_covered_by_disabled_decision(self) -> None:
        active = ET.Element(
            "testcase", {"classname": "Suite", "name": "Active", "status": "run"}
        )
        ET.SubElement(active, "failure", {"message": "regression"})
        disabled = source_disabled_rows([disabled_mapping()])
        cases = active_native_cases([active, disabled_xml()], disabled)
        with self.assertRaisesRegex(ValueError, "Unapproved"):
            check_native_cases(cases, {})


class SourceDisabledForcedRunTests(unittest.TestCase):
    def test_pinned_failure_is_recorded_without_pass_claim(self) -> None:
        disabled = source_disabled_rows([disabled_mapping()])
        observed = forced_disabled_observations([forced_xml()], disabled)
        self.assertFalse(observed[0]["legacy_assertions_passed"])
        self.assertEqual(
            observed[0]["current_behavior_rust_test"], "compatibility_retains_entries"
        )
        self.assertIn("actual: 8 vs 8", observed[0]["forced_failure_messages"][0])

    def test_missing_or_newly_passing_or_changed_failure_is_rejected(self) -> None:
        disabled = source_disabled_rows([disabled_mapping()])
        for cases in [
            [],
            [disabled_xml("run", "completed")],
            [forced_xml("new regression")],
        ]:
            with self.subTest(count=len(cases)), self.assertRaises(ValueError):
                forced_disabled_observations(cases, disabled)

    def test_forced_skip_and_duplicate_results_are_rejected(self) -> None:
        disabled = source_disabled_rows([disabled_mapping()])
        case = forced_xml()
        ET.SubElement(case, "skipped")
        for cases in [[case], [forced_xml(), forced_xml()], [disabled_xml()]]:
            with self.subTest(count=len(cases)), self.assertRaises(ValueError):
                forced_disabled_observations(cases, disabled)


class FakeEvidencePath:
    """Named in-memory filesystem fake for evidence persistence."""

    def __init__(self, name: str, contents: dict[str, str]) -> None:
        self.name = name
        self.contents = contents

    def __truediv__(self, name: str) -> "FakeEvidencePath":
        return FakeEvidencePath(self.name + "/" + name, self.contents)

    def __str__(self) -> str:
        return self.name

    def unlink(self, missing_ok: bool = False) -> None:
        self.contents.pop(self.name, None)

    def write_text(self, text: str) -> None:
        self.contents[self.name] = text

    def read_text(self) -> str:
        if self.name not in self.contents:
            raise FileNotFoundError(self.name)
        return self.contents[self.name]


class FakeXMLParser:
    """Read XML from the named evidence filesystem fake."""

    def __call__(self, path: FakeEvidencePath) -> ET.ElementTree:
        return ET.ElementTree(ET.fromstring(path.read_text()))


class FakeForcedNativeRun:
    """Named native-process fake writes XML or crashes before producing it."""

    def __init__(self, output: FakeEvidencePath, writes_xml: bool) -> None:
        self.output = output
        self.writes_xml = writes_xml
        self.command: list[str] = []

    def __call__(
        self, command: list[str], log: Path, environment: dict[str, str], approved: bool
    ) -> None:
        self.command = [str(part) for part in command]
        if self.writes_xml:
            xml = ET.Element("testsuite")
            xml.append(forced_xml())
            (self.output / "native-forced-disabled.xml").write_text(
                ET.tostring(xml, encoding="unicode")
            )


class SourceDisabledRunnerTests(unittest.TestCase):
    def test_forced_run_uses_exact_filter_and_writes_distinct_evidence(self) -> None:
        output = FakeEvidencePath("evidence", {})
        fake = FakeForcedNativeRun(output, True)
        with mock.patch.object(verify_toolkit, "run_native_suites", fake):
            with mock.patch.object(verify_toolkit.ET, "parse", FakeXMLParser()):
                verify_forced_disabled(
                    Path("oracle"),
                    output,
                    {},
                    source_disabled_rows([disabled_mapping()]),
                )
        self.assertIn("--gtest_also_run_disabled_tests", fake.command)
        self.assertIn("--gtest_filter=Suite.DISABLED_Original", fake.command)
        self.assertIn(
            '"legacy_assertions_passed": false',
            (output / "source-disabled.json").read_text(),
        )

    def test_stale_xml_cannot_hide_a_crashed_forced_run(self) -> None:
        output = FakeEvidencePath("evidence", {})
        (output / "native-forced-disabled.xml").write_text("<testsuite/>")
        fake = FakeForcedNativeRun(output, False)
        with mock.patch.object(verify_toolkit, "run_native_suites", fake):
            with mock.patch.object(verify_toolkit.ET, "parse", FakeXMLParser()):
                with self.assertRaises(FileNotFoundError):
                    verify_forced_disabled(
                        Path("oracle"),
                        output,
                        {},
                        source_disabled_rows([disabled_mapping()]),
                    )


class FakePinnedSource:
    """Named source-file fake supplies immutable bytes without filesystem I/O."""

    def __init__(self, content: bytes) -> None:
        self.content = content

    def __truediv__(self, relative: str) -> "FakePinnedSource":
        return self

    def read_bytes(self) -> bytes:
        return self.content


class DisabledSourceHashTests(unittest.TestCase):
    def test_disabled_mapping_still_requires_complete_unchanged_source(self) -> None:
        import hashlib

        content = b"TEST(Suite, DISABLED_Original) {}"
        row = disabled_mapping()
        row["source"] = "original.cxx"
        row["source_sha256"] = hashlib.sha256(content).hexdigest()
        self.assertEqual(
            verify_toolkit.verify_sources(FakePinnedSource(content), [row]),
            ["original.cxx"],
        )
        with self.assertRaisesRegex(ValueError, "Stale source hash"):
            verify_toolkit.verify_sources(FakePinnedSource(content + b"\n"), [row])
        with self.assertRaisesRegex(ValueError, "Incomplete source-suite"):
            verify_toolkit.verify_sources(
                FakePinnedSource(content + b" TEST(Suite, Active) {}"), [row]
            )


if __name__ == "__main__":
    unittest.main()
