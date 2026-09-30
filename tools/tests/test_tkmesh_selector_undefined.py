"""Selector null-mesh divergence inventory regression checks."""
import unittest
from verify_tkmesh_selector_undefined import candidates
from verify_tkmesh_numeric import corpus


class SelectorUndefinedTests(unittest.TestCase):
    def test_all_seven_dereferencing_overloads_remain_in_inventory(self) -> None:
        self.assertEqual(candidates(), [f'mesh_selector_invalid {op}' for op in range(7)])
        self.assertEqual(len(set(candidates())), 7)

    def test_undefined_queries_are_never_in_the_exact_corpus(self) -> None:
        requests = corpus().splitlines()
        self.assertTrue(set(candidates()).isdisjoint(requests))
        self.assertEqual(sum(row.startswith('mesh_selector ') for row in requests), 640)


if __name__ == '__main__':
    unittest.main()
