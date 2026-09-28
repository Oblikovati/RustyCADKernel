"""The Geom2dGridEval corpus must stay deterministic, reach every operation, and leave out
the B-spline DN parameters whose native evaluation reads past BSplCLib's arrays."""
import random
import unittest
from tkg2d_corpus import bspline, conic, offset, trimmed
from tkg2d_grid_corpus import append_tkg2d_grid_requests, dn_safe, flat_span, grid, grids, spline_under

POLES = [[0, 0, 1]] * 5
# Degree 1, knots 0, 0.3, 0.5, 0.6, 1 with end multiplicities 2: flat knots
# 0 0 0.3 0.5 0.6 1 1, flat spans 2 to 5, last knot index 5.
POLYLINE = bspline(1, [[0.0, 2], [0.3, 1], [0.5, 1], [0.6, 1], [1.0, 2]], POLES)


class SpanTests(unittest.TestCase):
    def test_flat_span_follows_locate_parameter(self):
        flat = [0.0, 0.0, 0.3, 0.5, 0.6, 1.0, 1.0]
        spans = [flat_span(flat, 1, 5, u) for u in (-1.0, 0.0, 0.3, 0.55, 0.6, 1.0, 2.0)]
        self.assertEqual(spans, [2, 2, 3, 4, 5, 5, 5])

    def test_dn_safe_drops_the_last_knot_index_span_only(self):
        self.assertEqual(dn_safe(POLYLINE, [0.0, 0.4, 0.55, 0.6, 0.9, 1.0]), [0.0, 0.4, 0.55])

    def test_dn_safe_reaches_wrapped_splines_and_keeps_other_curves(self):
        self.assertEqual(spline_under(offset(0.1, trimmed(0.0, 1.0, POLYLINE))), POLYLINE)
        self.assertIsNone(spline_under(conic(1, [0, 0], [1, 0], True, 2.0)))
        periodic = bspline(1, [[float(i), 1] for i in range(5)], POLES[:4], periodic=True)
        self.assertEqual(dn_safe(periodic, [3.5]), [3.5])


class GridTests(unittest.TestCase):
    def test_grid_writes_the_lower_bound_and_count(self):
        self.assertEqual(grid([0.5, 1.0], -3), [-3, 2, 0.5, 1.0])
        self.assertEqual(grid([]), [1, 0])

    def test_grids_end_with_the_empty_grid(self):
        sets = grids(random.Random(1), 0.0, 1.0, POLYLINE)
        self.assertEqual(sets[-1], [1, 0])
        self.assertEqual(len(sets), 6)


class CorpusTests(unittest.TestCase):
    def test_corpus_is_deterministic_and_covers_every_operation(self):
        first, second = [], []
        append_tkg2d_grid_requests(first)
        append_tkg2d_grid_requests(second)
        self.assertEqual(first, second)
        operations = {request.split()[0] for request in first}
        self.assertEqual(operations, {'g2_grid', 'g2_grid_adaptor', 'g2_grid_evalrep', 'g2_grid_null'})


if __name__ == '__main__':
    unittest.main()
