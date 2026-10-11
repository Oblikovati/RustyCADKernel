"""All plane visibility combinations and matching controls reach the native corpus."""

import unittest
from tkv3d_corpus_plane_presentations import append_plane_presentation_requests


class PlaneCorpusTests(unittest.TestCase):
    def test_all_branches_and_nonplane_controls_are_present(self) -> None:
        rows: list[str] = []
        append_plane_presentation_requests(rows)
        self.assertEqual(len(rows), 230)
        self.assertEqual(len(set(rows)), len(rows))
        flags = {
            int(row.split()[3]) for row in rows if row.startswith("v3_plane_add 0 0 ")
        }
        self.assertEqual(flags, set(range(16)))
        self.assertIn("v3_plane_add 0 1 0 1 1 0.5", rows)
        self.assertIn("v3_plane_add 0 2 0 1 1 0.5", rows)
        self.assertTrue(any(row.startswith("v3_plane_match 2 ") for row in rows))


if __name__ == "__main__":
    unittest.main()
