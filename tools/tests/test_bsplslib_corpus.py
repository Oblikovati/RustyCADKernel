"""The BSplSLib corpus must build valid surfaces and stay deterministic."""
import random
import unittest
from tkmath_bsplclib_curves import nb_poles
from tkmath_bsplclib_eval_corpus import last_u_knot_index
from tkmath_bsplslib_cache_corpus import index_range, schoenberg
from tkmath_bsplslib_corpus import append_bsplslib_requests
from tkmath_bsplslib_surfaces import encode_surface, first_u_knot_index, grid, surface


class SurfaceTests(unittest.TestCase):
    def test_grids_match_the_pole_counts(self):
        rng = random.Random(5)
        for case in range(200):
            definition = surface(rng, rng.randint(1, 6), rng.randint(1, 6), case % 2 == 0, case % 3 == 0,
                                 case % 5 == 0, case % 7 != 0)
            u, v = definition.u, definition.v
            self.assertEqual(definition.rows(), nb_poles(u.degree, u.periodic, u.mults))
            self.assertEqual(definition.cols(), nb_poles(v.degree, v.periodic, v.mults))
            self.assertEqual(len(definition.weights), definition.rows())

    def test_unclamped_directions_keep_a_valid_span(self):
        rng = random.Random(9)
        for _ in range(200):
            definition = surface(rng, rng.randint(1, 9), rng.randint(1, 9), clamped=False)
            for direction in (definition.u, definition.v):
                self.assertLess(first_u_knot_index(direction.degree, direction.mults),
                                last_u_knot_index(direction.degree, direction.mults))

    def test_first_u_knot_index(self):
        self.assertEqual(first_u_knot_index(3, [4, 1, 4]), 1)
        self.assertEqual(first_u_knot_index(3, [2, 1, 2, 4]), 3)

    def test_grid_encoding_is_row_major(self):
        self.assertEqual(grid(0, 2, [[1.0, 2.0], [3.0, 4.0]]), [0, 2, 2, 2, 1.0, 2.0, 3.0, 4.0])
        self.assertEqual(grid(1, 1, [[[1, 2, 3]]]), [1, 1, 1, 1, 1, 2, 3])

    def test_encoded_surface_header(self):
        definition = surface(random.Random(2), 2, 3, rational=True)
        encoded = encode_surface(definition, flat=True)
        self.assertEqual(encoded[:7], [2, 3, 0, 0, 1, 1, 0])

    def test_schoenberg_points_and_ranges(self):
        definition = surface(random.Random(4), 3, 2)
        self.assertEqual(len(schoenberg(definition.u)), definition.rows())
        first, last = index_range(random.Random(1), 5)
        self.assertTrue(1 <= first <= last <= 5)


class CorpusTests(unittest.TestCase):
    def test_corpus_is_deterministic(self):
        first, second = [], []
        append_bsplslib_requests(first)
        append_bsplslib_requests(second)
        self.assertEqual(first, second)
        self.assertTrue(all(request.startswith('bsps_') for request in first))


if __name__ == '__main__':
    unittest.main()
