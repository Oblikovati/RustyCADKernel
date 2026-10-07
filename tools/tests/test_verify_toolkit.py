"""The toolkit verifier: the numeric hang guard, declaration parsing and approved native failures."""
import inspect
import tempfile
import unittest
import xml.etree.ElementTree as ET
from pathlib import Path
from unittest import mock
import verify_toolkit
from verify_tkernel import run
from verify_toolkit import (NUMERIC_TIMEOUT_SECONDS, NUMERIC_VERIFIERS, approved_native_failures,
                            check_native_cases, declared_names, run_native_suites)


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

    def test_parameterized_declarations_are_found(self) -> None:
        # BRepGraph_Fuzz_Test.cxx expands each TEST_P over seven fixed seeds.
        text = 'TEST_P (Fuzz,\n Seed) {} TYPED_TEST_P(Typed, Case) {}'
        self.assertEqual(declared_names(text), {'Fuzz.Seed', 'Typed.Case'})

    def test_macro_suffixes_do_not_create_declarations(self) -> None:
        text = 'MY_TEST(Fake, Case) TESTING(Fake, Other) TEST_P(Real, Seed)'
        self.assertEqual(declared_names(text), {'Real.Seed'})



def gtest_case(name: str, outcome: str = 'pass') -> ET.Element:
    """One GoogleTest XML <testcase>: 'pass', 'failure' or 'skipped'."""
    suite, case = name.split('.')
    element = ET.Element('testcase', {'classname': suite, 'name': case, 'status': 'run'})
    if outcome != 'pass':
        ET.SubElement(element, outcome)
    return element


class ApprovedNativeFailureTests(unittest.TestCase):
    def test_only_the_approved_case_may_fail(self):
        cases = [gtest_case('S.A'), gtest_case('S.B', 'failure')]
        self.assertEqual(check_native_cases(cases, {'S.B': 'compile-time sinh'}), ['S.B'])
        with self.assertRaisesRegex(ValueError, 'Unapproved'):
            check_native_cases(cases, {})

    def test_an_approved_case_that_now_passes_fails_the_gate(self):
        with self.assertRaisesRegex(ValueError, 'now pass'):
            check_native_cases([gtest_case('S.A'), gtest_case('S.B')], {'S.B': 'why'})

    def test_skips_are_never_approved(self):
        with self.assertRaisesRegex(ValueError, 'Skipped'):
            check_native_cases([gtest_case('S.B', 'skipped')], {'S.B': 'why'})

    def test_approvals_come_from_ledger_rows_with_a_reason(self):
        rows = [{'upstream_id': 'S.A'}, {'upstream_id': 'S.B', 'native_expected_failure': 'why'}]
        self.assertEqual(approved_native_failures(rows), {'S.B': 'why'})
        with self.assertRaises(ValueError):
            approved_native_failures([{'upstream_id': 'S.B', 'native_expected_failure': ' '}])



class FailingGtestRun:
    """A run() stand-in for a GoogleTest binary that exits 1 because a test failed."""

    def __call__(self, command, log, env=None):
        raise RuntimeError('Command failed (1)')


class NativeExitCodeTests(unittest.TestCase):
    def test_exit_code_decides_only_without_approved_failures(self):
        with mock.patch.object(verify_toolkit, 'run', FailingGtestRun()):
            run_native_suites(['gtest'], Path('log'), {}, True)
            with self.assertRaises(RuntimeError):
                run_native_suites(['gtest'], Path('log'), {}, False)


class PassingCargoRun:
    """A run() stand-in for `cargo test` that reports the given cases as passed."""

    def __init__(self, names):
        self.log = ''.join(f'test {name} ... ok\n' for name in names)

    def __call__(self, command, log, env=None):
        return self.log


class RustTestIdentityTests(unittest.TestCase):
    # mapped_runtime_sets yields (package, target, test) identities, so rust_tests
    # must compare them per package and target.
    ROWS = [{'rust_package': 'pkg-a', 'rust_target': 'suite'}]

    def test_cases_of_other_packages_do_not_count(self):
        expected = {('pkg-a', 'suite', 'case_one'), ('pkg-b', 'suite', 'other')}
        with mock.patch.object(verify_toolkit, 'run', PassingCargoRun(['case_one'])):
            verify_toolkit.rust_tests(Path('out'), self.ROWS, expected, {})

    def test_a_missing_rust_case_is_refused(self):
        expected = {('pkg-a', 'suite', 'case_one'), ('pkg-a', 'suite', 'case_two')}
        with mock.patch.object(verify_toolkit, 'run', PassingCargoRun(['case_one'])):
            with self.assertRaisesRegex(ValueError, 'pkg-a/suite'):
                verify_toolkit.rust_tests(Path('out'), self.ROWS, expected, {})

    def test_a_port_only_test_in_a_mapped_target_is_refused_by_name(self):
        expected = {('pkg-a', 'suite', 'case_one')}
        with mock.patch.object(verify_toolkit, 'run', PassingCargoRun(['case_one', 'port_only'])):
            with self.assertRaisesRegex(ValueError, r"unmapped=\['port_only'\]"):
                verify_toolkit.rust_tests(Path('out'), self.ROWS, expected, {})


class FakeOcctSource:
    """A temporary OCCT tree holding one GTest source file."""

    def __init__(self, folder: Path, relative: str, text: str) -> None:
        self.root = folder
        (folder/relative).parent.mkdir(parents=True, exist_ok=True)
        (folder/relative).write_text(text)


class PendingNativeCaseTests(unittest.TestCase):
    SOURCE = 'src/TKA/GTests/A_Test.cxx'

    def pending_row(self, blocked_by):
        return {'upstream_id': 'A.Fused', 'source': self.SOURCE, 'source_sha256': 'b' * 64,
                'status': 'pending_dependency', 'blocked_by': blocked_by}

    def test_pending_rows_run_natively_only(self):
        with tempfile.TemporaryDirectory() as folder:
            occt = FakeOcctSource(Path(folder), self.SOURCE, 'TEST(A, Fused) { BRepAlgoAPI_Fuse f; }')
            cases = verify_toolkit.pending_native_cases(occt.root, [self.pending_row(['BRepAlgoAPI_Fuse'])])
        self.assertEqual(set(cases), {'A.Fused'})

    def test_a_blocker_the_source_never_names_is_refused(self):
        with tempfile.TemporaryDirectory() as folder:
            occt = FakeOcctSource(Path(folder), self.SOURCE, 'TEST(A, Fused) {}')
            with self.assertRaisesRegex(ValueError, 'never names'):
                verify_toolkit.pending_native_cases(occt.root, [self.pending_row(['BRepAlgoAPI_Fuse'])])


if __name__ == '__main__':
    unittest.main()
