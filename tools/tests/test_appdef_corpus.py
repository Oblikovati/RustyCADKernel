"""The AppDef corpus must stay deterministic, keep its request counts, and leave out the
paths whose native result reads uninitialized memory (the InitCutting simulation, the
unallocated vector arrays of a MultiPointConstraint)."""
import random
import unittest
from tkgeombase_corpus_appdef import (append_appdef_requests, assembly_size, elements_with_points,
                                      mpc_commands, mpc_spec, normal_part)
from tkgeombase_corpus_appdef_variational import C0, C2, PASS, TANGENT, CURVATURE, State, cutting_reads_unset_knot


class CuttingTests(unittest.TestCase):
    def test_one_element_never_reads_an_unset_knot(self):
        self.assertFalse(cutting_reads_unset_knot([0] * 10, 14, 2, 100))
        self.assertFalse(cutting_reads_unset_knot([], 14, 2, 100))

    def test_back_scan_overshoot_reads_the_knot_it_sets(self):
        # Eight tangency constraints (weight 2 each) over three free coefficients:
        # the back scan passes its count and compares with the unset knot.
        self.assertTrue(cutting_reads_unset_knot([1] * 8, 5, 2, 20))
        self.assertTrue(cutting_reads_unset_knot([2, 2, 2, 2], 8, 1, 10))

    def test_refused_cuttings_raise_before_any_knot(self):
        # A curvature constraint needs 3 coefficients: MaxDegree - Continuity = 2.
        self.assertFalse(cutting_reads_unset_knot([0, 2, 0], 4, 2, 5))


class StateTests(unittest.TestCase):
    def test_setters_rebuild_only_when_accepted(self):
        state = State([(1, PASS), (5, TANGENT), (9, PASS)], 10, 10, C2, 1)
        self.assertTrue(state.command_safe([10, 3]))
        self.assertEqual(state.maxseg, 3)
        self.assertTrue(state.command_safe([9, 1]))  # over-constrained: refused
        self.assertEqual(state.maxdeg, 10)
        self.assertTrue(state.command_safe([11, 7]))  # not a C0/C1/C2 continuity
        self.assertEqual(state.cont, C2)

    def test_set_constraints_replaces_the_couples(self):
        state = State([], 14, 100, C0, 0)
        self.assertTrue(state.command_safe([6, 2, 1, CURVATURE, 4, PASS]))
        self.assertEqual(state.kinds(), [2, 0])


class ContainerTests(unittest.TestCase):
    def test_curvatures_are_normal_to_the_tangents(self):
        c = normal_part([1.0, 2.0, 3.0], [0.0, 1.0, 0.0])
        self.assertEqual(c, [1.0, 0.0, 3.0])
        self.assertEqual(normal_part([1.0, 1.0], [0.0, 0.0]), [1.0, 1.0])

    def test_mpc_spec_reports_the_arrays_it_allocates(self):
        _, alloc, counts = mpc_spec(random.Random(1), 7, 1, 2, 3)
        self.assertEqual((alloc, counts), ({'t', 'c'}, (2, 0)))
        _, alloc, _ = mpc_spec(random.Random(1), 6, 1, 2, 3, mismatch=1)
        self.assertEqual(alloc, set())

    def test_mpc_commands_never_read_an_unallocated_array_in_range(self):
        # One 3D point without tangents: Tang(0) and Tang(2) raise, Tang(1) would
        # dereference the null array and is replaced by the flag query (9).
        _, c = mpc_commands(random.Random(2), set(), 1, 0)
        self.assertEqual(c[:6], [2, 0, 9, 9, 2, 2])
        _, c = mpc_commands(random.Random(2), {'t'}, 1, 0)
        self.assertEqual(c[:7], [2, 0, 2, 1, 2, 1, 2])

    def test_assembly_size_counts_shared_hermite_blocks(self):
        # Two C1 elements of degree 7 in 3 dimensions: 8 + 6 variables per dimension.
        self.assertEqual(assembly_size(3, 2, 7, 1), 42)

    def test_elements_with_points_includes_the_first_knot_of_element_one(self):
        self.assertEqual(elements_with_points([0.0, 0.5], [0.0, 0.1, 0.2, 1.0]), {1, 3})


class CorpusTests(unittest.TestCase):
    def test_corpus_is_deterministic_and_complete(self):
        first, second = [], []
        append_appdef_requests(first)
        append_appdef_requests(second)
        self.assertEqual(first, second)
        ops = {request.split()[0] for request in first}
        self.assertEqual(ops, {'ad_mpc', 'ad_line', 'ad_compute', 'ad_criteria', 'ad_var'})
        self.assertEqual(len(first), 555)


if __name__ == '__main__':
    unittest.main()
