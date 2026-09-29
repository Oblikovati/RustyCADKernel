"""The toolkit verifier gives the numeric verifier a hang guard well above the shared default."""
import inspect
import unittest
from verify_tkernel import run
from verify_toolkit import NUMERIC_TIMEOUT_SECONDS, NUMERIC_VERIFIERS, declared_names


class NumericTimeoutTests(unittest.TestCase):
    def test_numeric_verifier_has_its_own_longer_guard(self):
        # A full corpus must never be trimmed to fit the 900 s default of run().
        default = inspect.signature(run).parameters['timeout'].default
        self.assertGreaterEqual(NUMERIC_TIMEOUT_SECONDS, 4 * default)

    def test_every_toolkit_has_a_numeric_verifier(self):
        self.assertTrue({'TKMath', 'TKG2d', 'TKG3d', 'TKGeomBase'} <= set(NUMERIC_VERIFIERS))


class DeclaredNameTests(unittest.TestCase):
    def test_declarations_wrapped_after_the_comma_are_found(self):
        # BRepGraph_DeferredInvalidation_Test.cxx wraps one TEST_F after its suite name.
        text = 'TEST_F(Suite,\n       LongCase)\n{}\nTEST(Other, Short)\nTYPED_TEST( T , c )'
        self.assertEqual(declared_names(text), {'Suite.LongCase', 'Other.Short', 'T.c'})


if __name__ == '__main__':
    unittest.main()
