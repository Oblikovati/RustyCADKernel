"""Reject incomplete evidence and detect one-bit material regressions."""

import json
import unittest

from verify_renderer_material import compare_records, expected_cases, parse_records


def corpus() -> str:
    return "\n".join(
        json.dumps({"case": case, "bits": [0] * 56})
        for case in sorted(expected_cases())
    )


class RendererMaterialProofTests(unittest.TestCase):
    def test_complete_records_compare_exactly(self) -> None:
        self.assertEqual(len(parse_records(corpus())), 417)
        self.assertEqual(compare_records(corpus(), corpus()), [])

    def test_one_bit_change_is_failure(self) -> None:
        rows = corpus().splitlines()
        changed = json.loads(rows[0])
        changed["bits"][13] = 1
        rows[0] = json.dumps(changed)
        self.assertEqual(compare_records(corpus(), "\n".join(rows)), [changed["case"]])

    def test_missing_duplicate_and_renamed_cases_are_not_success(self) -> None:
        rows = corpus().splitlines()
        for invalid in (
            rows[:-1],
            [*rows, rows[0]],
            [rows[0].replace('"case": "', '"case": "invalid'), *rows[1:]],
        ):
            with self.subTest(count=len(invalid)), self.assertRaises(ValueError):
                parse_records("\n".join(invalid))

    def test_malformed_vectors_are_rejected(self) -> None:
        for bits in ([0] * 55, [False] * 56, [-1] * 56, [1 << 32] * 56, [0.0] * 56):
            with self.subTest(bits=bits[:1]), self.assertRaises(ValueError):
                parse_records(json.dumps({"case": "default", "bits": bits}))
