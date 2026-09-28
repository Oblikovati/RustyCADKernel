"""The BSplCLib corpus must build valid curves and stay deterministic."""
import random
import unittest
from tkmath_bsplclib_corpus import append_bsplclib_requests, sequence_ok
from tkmath_bsplclib_curves import curve, flat_knots, nb_poles
from tkmath_bsplclib_eval_corpus import last_u_knot_index


class CurveTests(unittest.TestCase):
    def test_periodic_flat_knots_extend_by_the_period(self):
        definition = curve(random.Random(1), 2, True)
        definition.knots, definition.mults = [0.0, 1.0, 2.0, 3.0], [1, 1, 1, 1]
        self.assertEqual(flat_knots(definition), [-2.0, -1.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0])

    def test_curves_have_enough_poles(self):
        rng = random.Random(7)
        for case in range(200):
            definition = curve(rng, rng.randint(1, 8), case % 2 == 0, clamped=case % 3 != 0)
            self.assertGreaterEqual(definition.count(), definition.degree+1)
            self.assertEqual(definition.count(), nb_poles(definition.degree, definition.periodic, definition.mults))
            self.assertTrue(sequence_ok(definition.degree, definition.periodic, definition.mults))

    def test_last_u_knot_index(self):
        self.assertEqual(last_u_knot_index(3, [4, 1, 4]), 3)
        self.assertEqual(last_u_knot_index(3, [4, 1, 2, 2]), 3)


class CorpusTests(unittest.TestCase):
    def test_corpus_is_deterministic(self):
        first, second = [], []
        append_bsplclib_requests(first)
        append_bsplclib_requests(second)
        self.assertEqual(first, second)
        self.assertTrue(all(request.startswith('bspl_') for request in first))


if __name__ == '__main__':
    unittest.main()
