"""The TKXSBase verifier wiring: a profile without numeric oracle, the native libraries it
links, its own closure tree."""
import unittest
from native_build import (EXCHANGE_CLOSURE_TARGETS, EXCHANGE_LINKED_TOOLKITS, EXCHANGE_TOOLKITS,
                          LINKED_TOOLKITS, OCAF_CLOSURE_TARGETS, closure_flags)
from verify_toolkit import NUMERIC_VERIFIERS, PROFILE_CLOSURES, PROFILES_WITHOUT_NUMERIC, linked_toolkits


class XsbaseVerifierProfileTests(unittest.TestCase):
    def test_tkxsbase_has_no_numeric_oracle(self):
        self.assertIn('TKXSBase', PROFILES_WITHOUT_NUMERIC)
        self.assertNotIn('TKXSBase', NUMERIC_VERIFIERS)

    def test_tkxsbase_links_its_externlib_closure_dependants_first(self):
        linked = linked_toolkits('TKXSBase')
        self.assertEqual(linked, EXCHANGE_LINKED_TOOLKITS)
        for name in ('TKBRep', 'TKernel', 'TKMath', 'TKG2d', 'TKG3d', 'TKTopAlgo', 'TKGeomBase', 'TKShHealing'):
            self.assertIn(name, linked)
        self.assertEqual(linked[0], 'TKXSBase')
        self.assertLess(linked.index('TKShHealing'), linked.index('TKTopAlgo'))
        self.assertEqual(linked[-1], 'TKernel')

    def test_the_exchange_closure_has_its_own_tree(self):
        toolkits, targets, tree = PROFILE_CLOSURES['TKXSBase']
        self.assertEqual(toolkits, EXCHANGE_TOOLKITS)
        self.assertEqual(tree, 'target/occt-exchange-full')
        self.assertEqual(targets, EXCHANGE_CLOSURE_TARGETS)
        self.assertEqual(PROFILE_CLOSURES['TKCDF'][1], OCAF_CLOSURE_TARGETS)
        self.assertIn('-DBUILD_ADDITIONAL_TOOLKITS=TKDE TKXSBase', closure_flags(['FoundationClasses'], EXCHANGE_TOOLKITS))
        self.assertNotIn('TKXSBase', LINKED_TOOLKITS)


if __name__ == '__main__':
    unittest.main()
