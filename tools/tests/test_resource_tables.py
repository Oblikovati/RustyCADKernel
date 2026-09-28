"""The Resource table generator must carry every literal and the corpus must stay defined."""
import struct
import unittest

from generate_resource_tables import encode, file_stem, fnv1a64, parse_arrays, rust_module
from tkernel_resource_corpus import boundary_cases, cases, utf8_encoder_cases, utf8_length


class ResourceTableGeneratorTests(unittest.TestCase):
    def test_arrays_parse_hex_and_decimal_literals_without_comment_digits(self):
        text = ("static const char16 unisjis[3] = { 0x0, 0x815f, // cp1250\n 12 };\n"
                "static char16 sjisuni[2] = { /* 99 */ 0x41, 0 };")
        self.assertEqual(parse_arrays(text), {"unisjis": [0, 0x815F, 12], "sjisuni": [0x41, 0]})

    def test_sparse_tables_keep_only_nonzero_entries_as_sorted_pairs(self):
        self.assertEqual(encode([0, 7, 0, 9], "sparse", "H"), struct.pack("<4H", 1, 7, 3, 9))
        self.assertEqual(encode([1, 0x2F9D4], "dense", "I"), struct.pack("<2I", 1, 0x2F9D4))

    def test_digest_is_fnv1a_64(self):
        self.assertEqual(fnv1a64(b""), 0xCBF29CE484222325)
        self.assertEqual(fnv1a64(b"a"), 0xAF63DC4C8601EC8C)

    def test_generated_module_wraps_long_items_like_rustfmt(self):
        module = rust_module([(file_stem("THE_CODEPAGES_ANSI"), "THE_CODEPAGES_ANSI", "h.pxx", "dense", 4, 1)])
        self.assertIn("pub(super) const CODE_PAGES: (&[u8], u64) =\n    (include_bytes!", module)


class ResourceCorpusTests(unittest.TestCase):
    def test_corpus_is_deterministic(self):
        self.assertEqual(cases(), cases())

    def test_utf8_length_follows_length_of_cstring(self):
        self.assertEqual(utf8_length([0x41, 0xE9, 0x20AC]), 6)
        self.assertEqual(utf8_length([0xD83D, 0xDE00, 0, 0x41]), 4)
        self.assertEqual(utf8_length([0xDC00]), 0)

    def test_undefined_native_writes_are_never_requested(self):
        for line in utf8_encoder_cases():
            _, _, size, units = line.split()
            decoded = [] if units == "-" else [int(units[i:i + 4], 16) for i in range(0, len(units), 4)]
            self.assertNotEqual(int(size), utf8_length(decoded))
        for line in boundary_cases():
            _, fmt, size, units = line.split()
            self.assertGreaterEqual(int(size), 1)
            if fmt in ("1", "3") and units[:4] in ("4e00", "3042"):
                self.assertGreater(int(size), 3)


if __name__ == "__main__":
    unittest.main()
