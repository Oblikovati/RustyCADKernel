"""Bounded private edge corpus and independent Cargo artifact identity."""
import unittest
from verify_tkbo_edge_helpers import EdgeHelperOracle, requests


class EdgeHelperCorpusTests(unittest.TestCase):
    def test_deterministic_complete_numeric_requests(self) -> None:
        first=requests()
        self.assertEqual(first,requests())
        self.assertEqual(len(first.splitlines()),2196)
        for row in first.splitlines():
            operation,*fields=row.split()
            values=[float(field) for field in fields]
            self.assertEqual(len(values),{'edge_math':8,'edge_geometry':6}[operation])
            if operation=='edge_math' and values[3]<0 and values[2]>0:
                self.assertLessEqual((values[1]-values[0])/values[2],10000)

    def test_probe_has_distinct_cargo_identity(self) -> None:
        oracle=EdgeHelperOracle('TKBO-edge-helpers','probe.cxx',[],'rustycad-tkbo',requests)
        self.assertEqual(oracle.rust_example,'rustycad-tkbo-edge-private-probe')
        self.assertNotEqual(oracle.rust_example,'rustycad-tkbo-numeric-probe')
