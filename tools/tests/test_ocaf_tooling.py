"""The TKCDF, TKLCAF and TKCAF verifier wiring: profiles without numeric oracle, the native
libraries they link, their own closure tree."""
import unittest
from native_build import (ADDITIONAL_TOOLKITS, CAF_CLOSURE_TARGETS, CAF_LINKED_TOOLKITS, CAF_TOOLKITS,
                          LINKED_TOOLKITS, OCAF_CLOSURE_TARGETS, OCAF_LINKED_TOOLKITS, OCAF_TOOLKITS,
                          closure_flags)
from verify_toolkit import NUMERIC_VERIFIERS, PROFILE_CLOSURES, PROFILES_WITHOUT_NUMERIC, linked_toolkits


class OcafVerifierProfileTests(unittest.TestCase):
    def test_ocaf_profiles_have_no_numeric_oracle(self):
        for toolkit in ('TKCDF', 'TKLCAF', 'TKCAF'):
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


if __name__ == '__main__':
    unittest.main()
