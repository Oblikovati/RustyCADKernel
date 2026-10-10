"""Ensure context placement stays in the full native verification corpus."""

import unittest
from tkv3d_corpus_context_location import append_location_requests
from verify_context_location import corpus


class ContextLocationCases(unittest.TestCase):
    def test_corpus_covers_every_flag_operation_and_placement(self) -> None:
        rows: list[str] = []
        append_location_requests(rows)
        self.assertEqual(len(rows), 1152)
        self.assertEqual(len(set(rows)), 1152)
        self.assertIn("v3_context_location 31 3 2.0 5.0", rows)
        self.assertIn("v3_context_location 0 0 0.0 0.0", rows)
        self.assertEqual(corpus(), "\n".join(rows) + "\n")


if __name__ == "__main__":
    unittest.main()
