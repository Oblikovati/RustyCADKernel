"""The TKDE verifier wiring: a profile without numeric oracle, the DE format libraries its
unchanged suite links, its own closure tree, and the CI step that runs it."""
import unittest
from pathlib import Path
from native_build import DE_CLOSURE_TARGETS, DE_LINKED_TOOLKITS, DE_TOOLKITS, closure_flags
from verify_toolkit import NUMERIC_VERIFIERS, PROFILE_CLOSURES, PROFILES_WITHOUT_NUMERIC, linked_toolkits

ROOT = Path(__file__).resolve().parents[2]


class TkdeVerifierProfileTests(unittest.TestCase):
    def test_tkde_has_no_numeric_oracle(self):
        self.assertIn('TKDE', PROFILES_WITHOUT_NUMERIC)
        self.assertNotIn('TKDE', NUMERIC_VERIFIERS)

    def test_tkde_links_every_format_before_the_framework(self):
        linked = linked_toolkits('TKDE')
        self.assertEqual(linked, DE_LINKED_TOOLKITS)
        # DE_Wrapper_Configuration_Test binds DEBREP and DEXCAF (TKDECascade) and the format nodes.
        for name in ('TKDECascade', 'TKDEGLTF', 'TKDESTEP', 'TKDEIGES', 'TKDE'):
            self.assertIn(name, linked)
        self.assertLess(linked.index('TKDESTEP'), linked.index('TKDE'))
        self.assertEqual(linked[-1], 'TKernel')

    def test_tkde_builds_its_own_closure_tree(self):
        toolkits, targets, tree = PROFILE_CLOSURES['TKDE']
        self.assertEqual(toolkits, DE_TOOLKITS)
        self.assertEqual(targets, [*DE_TOOLKITS, 'gtest_main'])
        self.assertEqual(targets, DE_CLOSURE_TARGETS)
        self.assertNotIn(tree, {closure[2] for name, closure in PROFILE_CLOSURES.items() if name != 'TKDE'})
        self.assertIn('-DBUILD_ADDITIONAL_TOOLKITS=' + ' '.join(DE_TOOLKITS), closure_flags(['FoundationClasses'], DE_TOOLKITS))

    def test_ci_runs_the_tkde_profile(self):
        workflow = (ROOT/'.github/workflows/ci.yml').read_text()
        self.assertIn('python3 tools/verify_toolkit.py TKDE --source target/occt-source --jobs 2', workflow)


if __name__ == '__main__':
    unittest.main()
