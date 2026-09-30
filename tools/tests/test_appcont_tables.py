"""The AppCont table generator must copy literals verbatim and refuse unexpected shapes."""
import subprocess
import tempfile
import unittest
from pathlib import Path

from tkgeombase_corpus_appcont import append_appcont_requests
from toolkit_numeric import NumericOracle, check_generators
from verify_tkgeombase_numeric import ORACLE
from generate_appcont_tables import CHUNK, PER_LINE, TABLES, chunk_order, fragments, parse_table, rust_name


class AppContTableGeneratorTests(unittest.TestCase):
    def test_literals_keep_text_including_folded_differences(self):
        text = ('// header\nnamespace A {\n//! note 12, 13\nstatic const double THE_X_DATA[] = {\n'
                '  0.1666666666666666666666667,\n  1.332e18 - 2.211e17,\n  -5.5e23, /* 7.0 */ 2.873e-7\n};\n}')
        self.assertEqual(parse_table(text, 'THE_X_DATA'),
                         ['0.1666666666666666666666667', '1.332e18 - 2.211e17', '-5.5e23', '2.873e-7'])
        with self.assertRaises(ValueError):
            # Only the literal shapes of the native tables are accepted (all carry a decimal point).
            parse_table(text.replace('2.873e-7', '2873e-10'), 'THE_X_DATA')

    def test_unexpected_literals_and_missing_arrays_are_refused(self):
        with self.assertRaises(ValueError):
            parse_table('static const double THE_X_DATA[] = { 1.0 * 2.0 };', 'THE_X_DATA')
        with self.assertRaises(ValueError):
            parse_table('static const double THE_X_DATA[] = { 3 };', 'THE_X_DATA')
        with self.assertRaises(ValueError):
            parse_table('static const double THE_Y_DATA[] = { 3.0 };', 'THE_X_DATA')

    def test_fragments_pack_literals_per_line_under_the_chunk_size(self):
        values = [f'{i}.5' for i in range(CHUNK + 5)]
        files = fragments('THE_VBMATRIX_DATA', values)
        self.assertEqual(sorted(files, key=chunk_order), ['vbmatrix_0.rs', 'vbmatrix_1.rs'])
        self.assertEqual(files['vbmatrix_1.rs'], f'&[\n    {", ".join(values[CHUNK:CHUNK + PER_LINE])},\n'
                                                 f'    {values[-1]},\n]\n')
        self.assertLess(files['vbmatrix_0.rs'].count('\n'), 500)

    def test_names_and_expected_sizes_follow_the_native_block_layout(self):
        self.assertEqual(rust_name('THE_INV_MMATRIX_DATA'), 'inv_mmatrix')
        self.assertEqual({name: size for name, _, size in TABLES},
                         {'THE_BBMATRIX_DATA': 6200, 'THE_INV_MMATRIX_DATA': 6200, 'THE_IBPMATRIX_DATA': 4900,
                          'THE_IBTMATRIX_DATA': 3795, 'THE_VBMATRIX_DATA': 105000})


class GeneratorCheckTests(unittest.TestCase):
    def test_tkgeombase_oracle_checks_the_appcont_tables(self):
        self.assertIn('tools/generate_appcont_tables.py', ORACLE.generators)

    def test_a_drifted_generator_fails_the_numeric_verification(self):
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            for name, code in (('clean_gen.py', 0), ('stale_gen.py', 1)):
                (folder/name).write_text(f'import sys\nsys.exit({code})\n')
            clean = NumericOracle('X', 'p.cxx', [], 'x', str, generators=(str(folder/'clean_gen.py'),))
            check_generators(clean, folder, folder)
            self.assertTrue((folder/'clean_gen-check.log').is_file())
            stale = NumericOracle('X', 'p.cxx', [], 'x', str, generators=(str(folder/'stale_gen.py'),))
            with self.assertRaises(subprocess.CalledProcessError):
                check_generators(stale, folder, folder)


class AppContCorpusTests(unittest.TestCase):
    def test_corpus_is_deterministic_and_reads_every_table_entry(self):
        first, second = [], []
        append_appcont_requests(first)
        append_appcont_requests(second)
        self.assertEqual(first, second)
        self.assertEqual({request.split()[0] for request in first}, {'acc_table', 'acc_offsets', 'acc_matrix', 'acc_ls'})
        read = {}
        for request in first:
            if request.startswith('acc_table '):
                kind, start, count = map(int, request.split()[1:])
                read[kind] = read.get(kind, 0) + count
        self.assertEqual(read, {kind: size for kind, (_, _, size) in enumerate(TABLES)})

    def test_native_undefined_behavior_is_never_requested(self):
        requests = []
        append_appcont_requests(requests)
        for request in requests:
            fields = request.split()
            if fields[0] == 'acc_matrix' and fields[1] == '4':
                self.assertFalse(fields[2] == '1' and int(fields[3]) >= 1, request)
            if fields[0] == 'acc_ls':
                degree, nbpoints = int(fields[-2]), int(fields[-1])
                # Degree -1 and 0 Gauss points build empty math_Matrix ranges, which OCCT
                # accepts (the empty-bounds port); degree -2 crashes natively and more than
                # 61 Gauss points read past the tables, so those are never requested.
                self.assertTrue(degree >= -1 and 0 <= nbpoints <= 61, request)


if __name__ == '__main__':
    unittest.main()
