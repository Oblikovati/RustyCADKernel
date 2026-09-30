"""Regression checks for explicit Delaun divergence inventory publication."""
import json
import sys
import unittest
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from verify_tkmesh_delaun_undefined import candidates, write_report


class RecordingEvidenceDirectory:
    """Named in-memory evidence sink: no filesystem writes."""
    def __init__(self) -> None:
        self.files: dict[str, str] = {}

    def __truediv__(self, filename: str) -> 'RecordingEvidenceFile':
        return RecordingEvidenceFile(self.files, filename)


class RecordingEvidenceFile:
    """One in-memory report artifact."""
    def __init__(self, files: dict[str, str], filename: str) -> None:
        self.files = files
        self.filename = filename

    def write_text(self, text: str) -> int:
        self.files[self.filename] = text
        return len(text)


class DelaunUndefinedTests(unittest.TestCase):
    def test_all_discovered_reads_and_writes_remain_in_inventory(self) -> None:
        rows = candidates()
        self.assertEqual(len(rows), 8)
        self.assertEqual(len(set(rows)), 8)
        self.assertIn('mesh_delaun_invalid 0 -1', rows)
        self.assertIn('mesh_delaun_invalid 1 2', rows)
        self.assertEqual(rows[-2:], ['mesh_delaun_invalid 2 1', 'mesh_delaun_invalid 3 1'])

    def test_report_never_counts_undefined_cases_as_exact_passes(self) -> None:
        output = RecordingEvidenceDirectory()
        records = [{'input': row} for row in candidates()]
        write_report(output, records)
        report = json.loads(output.files['report.json'])
        self.assertEqual(report['candidates'], 8)
        self.assertEqual(report['exact_passed'], 0)
        self.assertFalse(report['complete_parity'])
        self.assertEqual(report['undefined_index_divergences'], records)
        self.assertEqual(output.files['inputs.txt'].splitlines(), candidates())


if __name__ == '__main__':
    unittest.main()
