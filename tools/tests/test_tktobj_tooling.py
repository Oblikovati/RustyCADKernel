"""The TKTObj verifier wiring: a profile without numeric oracle that links the BinTObj
format toolkit its unchanged suite registers, in its own closure tree."""
import unittest
from native_build import TOBJ_CLOSURE_TARGETS, TOBJ_LINKED_TOOLKITS, TOBJ_TOOLKITS, closure_flags
from verify_toolkit import NUMERIC_VERIFIERS, PROFILE_CLOSURES, PROFILES_WITHOUT_NUMERIC, linked_toolkits


class TObjVerifierProfileTests(unittest.TestCase):
    def test_tktobj_has_no_numeric_oracle(self):
        self.assertIn('TKTObj', PROFILES_WITHOUT_NUMERIC)
        self.assertNotIn('TKTObj', NUMERIC_VERIFIERS)

    def test_tktobj_links_the_bintobj_format_before_its_own_toolkit(self):
        linked = linked_toolkits('TKTObj')
        self.assertEqual(linked, TOBJ_LINKED_TOOLKITS)
        # Dependants come first: TKBinTObj sits on TKBinL and TKTObj, which sit on TKLCAF.
        self.assertLess(linked.index('TKBinTObj'), linked.index('TKBinL'))
        self.assertLess(linked.index('TKBinL'), linked.index('TKLCAF'))
        self.assertLess(linked.index('TKTObj'), linked.index('TKLCAF'))
        self.assertLess(linked.index('TKLCAF'), linked.index('TKCDF'))
        self.assertEqual(linked[-1], 'TKernel')

    def test_tktobj_builds_its_own_closure_tree(self):
        toolkits, targets, tree = PROFILE_CLOSURES['TKTObj']
        self.assertEqual((toolkits, targets, tree),
                         (TOBJ_TOOLKITS, TOBJ_CLOSURE_TARGETS, 'target/occt-tobj-full'))
        self.assertEqual(TOBJ_CLOSURE_TARGETS, ['TKBinTObj', 'gtest_main'])
        self.assertIn('-DBUILD_ADDITIONAL_TOOLKITS=TKBinTObj', closure_flags(['FoundationClasses'], TOBJ_TOOLKITS))
        trees = [closure[2] for name, closure in PROFILE_CLOSURES.items() if name != 'TKTObj']
        self.assertNotIn(tree, trees)


if __name__ == '__main__':
    unittest.main()
