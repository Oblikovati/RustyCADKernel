"""No radius candidate disappears or becomes a false exact-parity claim."""
import unittest
from tkbo_algorithms_corpus import algorithm_candidates,algorithm_partition
from tkbo_undefined_radius import validate_radius_observation,RADIUS_SENTINEL


class RadiusAccountingTests(unittest.TestCase):
    def test_all_candidates_are_retained_in_disjoint_partitions(self) -> None:
        candidates=algorithm_candidates();exact,undefined=algorithm_partition()
        self.assertEqual(len(candidates),2736)
        self.assertEqual(len(undefined),176)
        self.assertEqual(set(candidates),set(exact)|set(undefined))
        self.assertFalse(set(exact)&set(undefined))

    def test_incidental_equality_is_recorded_as_undefined(self) -> None:
        row=validate_radius_observation('request','1 0 0 1 19 0 0 0 0','1 0 0 1 19 0 0 0 0')
        self.assertFalse(row['observed_difference'])
        self.assertIn('never an exact',row['classification'])

    def test_native_failure_retains_the_old_radius(self) -> None:
        row=validate_radius_observation('request',f'1 -1 {RADIUS_SENTINEL} 1 19 0 0 0 0','1 0 0 1 19 0 0 0 0')
        self.assertTrue(row['observed_difference'])

    def test_unrelated_difference_is_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError,'only status/radius'):
            validate_radius_observation('request','2 0 0 1 19 0 0 0 0','1 0 0 1 19 0 0 0 0')
