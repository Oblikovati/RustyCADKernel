"""Private contour corpus remains deterministic and has a distinct Cargo identity."""
import unittest
from verify_tkhlr_contour_helpers import ContourHelperOracle, requests


class ContourHelperCorpusTests(unittest.TestCase):
    def test_requests_are_repeatable_numeric_and_complete(self):
        first = requests()
        self.assertEqual(first, requests())
        self.assertEqual(len(first.splitlines()), 576)
        for line in first.splitlines():
            self.assertEqual(line.split()[0], 'contour_math')
            self.assertEqual(len([float(value) for value in line.split()[1:]]), 26)

    def test_private_example_cannot_reuse_public_probe(self):
        oracle = ContourHelperOracle('TKHLR-private', 'probe.cxx', [], 'rustycad-tkhlr', requests)
        self.assertEqual(oracle.rust_example, 'rustycad-tkhlr-contour-private-probe')
        self.assertNotEqual(oracle.rust_example, oracle.rust_package+'-numeric-probe')
