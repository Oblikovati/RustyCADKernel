"""The Adaptor2d/Geom2dAdaptor corpus must stay deterministic and reach every adaptor kind."""
import random
import unittest
from tkg2d_adaptor_corpus import (BASE, a_geom, a_line, a_offset, append_tkg2d_adaptor_requests, knotted_bspline)


class SpecTests(unittest.TestCase):
    def test_adaptor_specs_follow_the_probe_grammar(self):
        self.assertEqual(a_line([0, 0], [1, 0], 0.0, 1.0), [0, 0, 0, 1, 0, 0.0, 1.0])
        self.assertEqual(a_geom([0, 0, 0, 1, 0], [0.2, 0.4]), [4, 0.2, 0.4, 0, 0, 0, 1, 0])
        self.assertEqual(a_offset(BASE), [5, 8])
        self.assertEqual(a_offset(BASE, 1.5), [6, 1.5, 8])
        self.assertEqual(a_offset(BASE, 1.5, [0.0, 2.0]), [7, 1.5, 0.0, 2.0, 8])

    def test_knotted_bspline_has_one_pole_per_free_knot(self):
        spec = knotted_bspline(random.Random(3))
        knot_count = spec[4]
        multiplicities = spec[6:6 + 2 * knot_count:2]
        self.assertEqual(spec[5 + 2 * knot_count], sum(multiplicities) - 3 - 1)


class CorpusTests(unittest.TestCase):
    def test_corpus_is_deterministic_and_covers_every_operation(self):
        first, second = [], []
        append_tkg2d_adaptor_requests(first)
        append_tkg2d_adaptor_requests(second)
        self.assertEqual(first, second)
        operations = {request.split()[0] for request in first}
        self.assertEqual(operations, {'a2_adaptor', 'a2_load', 'a2_offset_load', 'a2_evalrep'})
        kinds = {int(request.split()[1]) for request in first if request.startswith('a2_adaptor')}
        self.assertEqual(kinds, {0, 1, 2, 3, 4, 5, 6, 7, 8})


if __name__ == '__main__':
    unittest.main()
