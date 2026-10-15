"""The TKCDF, TKLCAF, TKCAF, TKVCAF and TKXCAF verifier wiring: profiles without numeric oracle, the native
libraries they link, their own closure tree."""
import unittest
from native_build import (ADDITIONAL_TOOLKITS, CAF_CLOSURE_TARGETS, CAF_LINKED_TOOLKITS, CAF_TOOLKITS,
                          LINKED_TOOLKITS, OCAF_CLOSURE_TARGETS, OCAF_LINKED_TOOLKITS, OCAF_TOOLKITS,
                          V3D_LINKED_TOOLKITS, VCAF_CLOSURE_TARGETS, VCAF_LINKED_TOOLKITS,
                          XCAF_CLOSURE_TARGETS, XCAF_LINKED_TOOLKITS, closure_flags)
from verify_toolkit import NUMERIC_VERIFIERS, PROFILE_CLOSURES, PROFILES_WITHOUT_NUMERIC, linked_toolkits


class OcafVerifierProfileTests(unittest.TestCase):
    def test_ocaf_profiles_have_no_numeric_oracle(self):
        for toolkit in ('TKCDF', 'TKLCAF', 'TKCAF', 'TKVCAF', 'TKXCAF'):
            self.assertIn(toolkit, PROFILES_WITHOUT_NUMERIC)
            self.assertNotIn(toolkit, NUMERIC_VERIFIERS)

    def test_tkcdf_links_only_its_own_closure(self):
        self.assertEqual(linked_toolkits('TKCDF'), ('TKCDF', 'TKernel'))

    def test_tklcaf_links_the_toolkits_its_unchanged_suites_include(self):
        linked = linked_toolkits('TKLCAF')
        for name in ('TKBin', 'TKCAF', 'TKPrim', 'TKLCAF', 'TKCDF'):
            self.assertIn(name, linked)
        # Dependants come first: TKBin sits on TKCAF, which sits on TKLCAF and TKCDF.
        self.assertLess(linked.index('TKBin'), linked.index('TKCAF'))
        self.assertLess(linked.index('TKCAF'), linked.index('TKLCAF'))
        self.assertLess(linked.index('TKLCAF'), linked.index('TKCDF'))
        self.assertEqual(linked[-1], 'TKernel')

    def test_geometry_profiles_keep_the_shared_closure(self):
        self.assertEqual(linked_toolkits('TKMath'), LINKED_TOOLKITS)
        self.assertNotIn('TKLCAF', LINKED_TOOLKITS)

    def test_the_ocaf_closure_is_separate_from_the_geometry_one(self):
        self.assertEqual(OCAF_CLOSURE_TARGETS, [*OCAF_TOOLKITS, 'gtest_main'])
        self.assertIn('-DBUILD_ADDITIONAL_TOOLKITS=TKBin TKCAF TKPrim', closure_flags(['FoundationClasses'], OCAF_TOOLKITS))
        self.assertIn('-DBUILD_ADDITIONAL_TOOLKITS=' + ' '.join(ADDITIONAL_TOOLKITS), closure_flags(['FoundationClasses']))
        self.assertEqual(set(OCAF_TOOLKITS) - set(OCAF_LINKED_TOOLKITS), set())

    def test_tkcaf_links_the_draw_naming_toolkit_before_the_ocaf_closure(self):
        linked = linked_toolkits('TKCAF')
        self.assertEqual(linked, CAF_LINKED_TOOLKITS)
        # TNaming_Tool_Test drives DNaming (TKDCAF), whose drivers sit on TKBool and TKFillet.
        for name in ('TKDCAF', 'TKBool', 'TKFillet', 'TKCAF', 'TKBO', 'TKPrim'):
            self.assertIn(name, linked)
        self.assertLess(linked.index('TKDCAF'), linked.index('TKCAF'))
        self.assertLess(linked.index('TKBool'), linked.index('TKBO'))
        self.assertEqual(linked[-1], 'TKernel')

    def test_tkcaf_builds_its_own_closure_tree(self):
        toolkits, targets, tree = PROFILE_CLOSURES['TKCAF']
        self.assertEqual(toolkits, CAF_TOOLKITS)
        self.assertEqual(targets, [*CAF_TOOLKITS, 'gtest_main'])
        self.assertEqual(targets, CAF_CLOSURE_TARGETS)
        self.assertNotEqual(tree, PROFILE_CLOSURES['TKLCAF'][2])
        self.assertEqual(set(OCAF_TOOLKITS) - set(CAF_TOOLKITS), set())
        self.assertEqual(set(CAF_TOOLKITS) - set(CAF_LINKED_TOOLKITS), set())

    def test_tkvcaf_links_the_ocaf_and_visualization_closures(self):
        linked = linked_toolkits('TKVCAF')
        self.assertEqual(linked, VCAF_LINKED_TOOLKITS)
        # TPrsStd sits on TKCAF (TDataXtd) and TKV3d (AIS); every TKV3d dependency is linked.
        self.assertEqual(set(V3D_LINKED_TOOLKITS) - set(linked), set())
        self.assertLess(linked.index('TKVCAF'), linked.index('TKCAF'))
        self.assertLess(linked.index('TKCAF'), linked.index('TKBO'))
        self.assertLess(linked.index('TKLCAF'), linked.index('TKCDF'))
        self.assertEqual(linked[-1], 'TKernel')

    def test_tkxcaf_adds_itself_to_the_tkvcaf_closure(self):
        self.assertEqual(linked_toolkits('TKXCAF'), ('TKXCAF', *VCAF_LINKED_TOOLKITS))
        self.assertIn('TKPrim', XCAF_LINKED_TOOLKITS)

    def test_visualization_ocaf_profiles_build_their_own_trees(self):
        self.assertEqual(PROFILE_CLOSURES['TKVCAF'][1], VCAF_CLOSURE_TARGETS)
        self.assertEqual(PROFILE_CLOSURES['TKXCAF'][1], XCAF_CLOSURE_TARGETS)
        trees = {PROFILE_CLOSURES[name][2] for name in ('TKCAF', 'TKV3d', 'TKVCAF', 'TKXCAF')}
        self.assertEqual(len(trees), 4)


if __name__ == '__main__':
    unittest.main()
