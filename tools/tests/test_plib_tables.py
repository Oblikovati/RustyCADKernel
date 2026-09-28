"""The Jacobi table generator must copy literals verbatim and prove the prefix property."""
import unittest
from generate_plib_jacobi_tables import degree30_coefficients, fragments, parse_tables, rust_name
from tkmath_plib_corpus import append_plib_requests


class TableGeneratorTests(unittest.TestCase):
    def test_literals_keep_text_and_skip_row_markers(self):
        text = ('static constexpr double TransMatrix_C0[2][2] = {\n  {0.1e+01, -0.2e+00},\n'
                '  // TransMatrix_C0[1][0]\n  {0.3e+01, 0.4e+00}};\n')
        self.assertEqual(parse_tables(text)['TransMatrix_C0'], ['0.1e+01', '-0.2e+00', '0.3e+01', '0.4e+00'])

    def test_declared_shape_must_match_literal_count(self):
        with self.assertRaises(ValueError):
            parse_tables('static constexpr double MaxValuesDB_C0[3] = {1.0, 2.0};')

    def test_coefficients_that_are_not_prefixes_are_refused(self):
        tables = {f'{kind}_C{level}_D{degree}': ['1.0']*(degree+1)
                  for kind in ['TNorm', 'CofA', 'CofB', 'Denom'] for level in range(3) for degree in range(31)}
        self.assertEqual(len(degree30_coefficients(tables)['CofA_C2']), 31)
        tables['Denom_C1_D7'][3] = '2.0'
        with self.assertRaises(ValueError):
            degree30_coefficients(tables)

    def test_fragments_are_slice_expressions_named_like_rust_constants(self):
        self.assertEqual(rust_name('WeightsDB0_C1'), 'WEIGHTS_DB0_C1')
        files = fragments('MaxValuesDB_C0', ['1.5', '-2.5'], '// header\n')
        self.assertEqual(files, {'max_values_db_c0_0.rs': '// header\n&[\n    1.5,\n    -2.5,\n]\n'})


class PlibCorpusTests(unittest.TestCase):
    def test_corpus_is_deterministic_and_covers_every_operation(self):
        first, second = [], []
        append_plib_requests(first)
        append_plib_requests(second)
        self.assertEqual(first, second)
        operations = {request.split()[0] for request in first}
        self.assertEqual(len(operations), 14)


if __name__ == '__main__':
    unittest.main()
