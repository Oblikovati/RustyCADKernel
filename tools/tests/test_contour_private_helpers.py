"""Private contour corpus remains deterministic and has a distinct Cargo identity."""
import unittest
from verify_tkhlr_contour_helpers import ContourHelperOracle, requests


class ContourHelperCorpusTests(unittest.TestCase):
    def test_requests_are_repeatable_numeric_and_complete(self):
        first = requests()
        self.assertEqual(first, requests())
        self.assertEqual(len(first.splitlines()), 1224)
        for line in first.splitlines():
            operation = line.split()[0]
            expected = {'contour_math': 26, 'contour_sections': 21, 'contour_attachment': 21}
            self.assertIn(operation, expected)
            self.assertEqual(len([float(value) for value in line.split()[1:]]), expected[operation])

    def test_private_example_cannot_reuse_public_probe(self):
        oracle = ContourHelperOracle('TKHLR-private', 'probe.cxx', [], 'rustycad-tkhlr', requests)
        self.assertEqual(oracle.rust_example, 'rustycad-tkhlr-contour-private-probe')
        self.assertNotEqual(oracle.rust_example, oracle.rust_package+'-numeric-probe')
