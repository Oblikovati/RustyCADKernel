"""Complete deterministic BeanFace helper corpus and a distinct Cargo target."""
import unittest
from verify_tkbo_bean_helpers import BeanHelperOracle,requests


class BeanHelperCorpusTests(unittest.TestCase):
    def test_all_private_request_shapes_are_deterministic(self) -> None:
        rows=requests();self.assertEqual(rows,requests());self.assertEqual(len(rows.splitlines()),1017)
        for row in rows.splitlines():
            operation,*values=row.split()
            self.assertEqual(len(values),{'bean_empty':6,'bean_group':3,'bean_sampling':12}[operation])
            if operation=='bean_group':self.assertGreater(int(values[2]),0)

    def test_private_probe_uses_an_independent_cargo_artifact(self) -> None:
        oracle=BeanHelperOracle('TKBO-bean-helpers','probe.cxx',[],'rustycad-tkbo',requests)
        self.assertEqual(oracle.rust_example,'rustycad-tkbo-bean-private-probe')
        self.assertNotEqual(oracle.rust_example,'rustycad-tkbo-numeric-probe')
