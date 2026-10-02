"""Complete deterministic BeanFace helper corpus and a distinct Cargo target."""
import unittest
from verify_tkbo_bean_helpers import BeanHelperOracle,requests
from tkbo_bean_private_corpus import fast_analytic_requests, grid_requests, line_plane_requests


class BeanHelperCorpusTests(unittest.TestCase):
    def test_all_private_request_shapes_are_deterministic(self) -> None:
        rows=requests();self.assertEqual(rows,requests());self.assertEqual(len(rows.splitlines()),5661)
        for row in rows.splitlines():
            operation,*values=row.split()
            self.assertEqual(len(values),{'bean_box':9,'bean_empty':6,'bean_fast_analytic':19,'bean_grid':8,'bean_group':3,'bean_line_plane':16,'bean_sampling':12}[operation])
            if operation=='bean_group':self.assertGreater(int(values[2]),0)
        self.assertEqual(len(grid_requests()),864)
        self.assertEqual(len(line_plane_requests()),972)
        self.assertEqual(len(fast_analytic_requests()),1728)

    def test_private_probe_uses_an_independent_cargo_artifact(self) -> None:
        oracle=BeanHelperOracle('TKBO-bean-helpers','probe.cxx',[],'rustycad-tkbo',requests)
        self.assertEqual(oracle.rust_example,'rustycad-tkbo-bean-private-probe')
        self.assertNotEqual(oracle.rust_example,'rustycad-tkbo-numeric-probe')
