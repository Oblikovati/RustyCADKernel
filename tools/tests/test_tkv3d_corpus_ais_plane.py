"""Guard native plane corpus coverage and undefined detached-drawer preconditions."""

import unittest
from tkv3d_corpus_ais_plane import append_ais_plane_requests


class AisPlaneCorpusTests(unittest.TestCase):
    def test_all_constructor_geometry_and_drawer_variants_are_exercised(self) -> None:
        requests: list[str] = []
        append_ais_plane_requests(requests)
        self.assertEqual(len(requests), 210)
        variants = {tuple(row.split()[1:4]) for row in requests}
        self.assertEqual(len(variants), 42)

    def test_operation_counts_and_detached_aspect_preconditions(self) -> None:
        requests: list[str] = []
        append_ais_plane_requests(requests)
        for row in requests:
            fields = row.split()
            operations = tuple(map(int, fields[5:]))
            self.assertEqual(int(fields[4]), len(operations))
            if fields[3] == "0":
                self.assertNotIn(6, operations)
                self.assertNotIn(8, operations)
        linked_operations = {
            op
            for row in requests
            if row.split()[3] == "1"
            for op in map(int, row.split()[5:])
        }
        self.assertEqual(linked_operations, set(range(24)) - {2})


if __name__ == "__main__":
    unittest.main()
