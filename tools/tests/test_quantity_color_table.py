"""The Quantity color-table generator copies upstream literals and enum aliases verbatim."""
import tempfile
import unittest
from pathlib import Path

import generate_quantity_color_table as table
import tkernel_quantity_corpus as corpus

ENUM = """
enum Quantity_NameOfColor
{
  Quantity_NOC_BLACK,
  Quantity_NOC_BLUE, //!< comment
  Quantity_NOC_BLUE1 = Quantity_NOC_BLUE,
  Quantity_NOC_WHITE
};
"""
PXX = """RawColor(BLACK, sRGB, 0x000000, 0.000000, 0.000000, 0.000000, RGB, 0.000000, 0.000000, 0.000000),
  RawColor(BLUE,
           sRGB, 0x0000FF, 0.000000, 0.000000, 1.000000,
           RGB, 0.000000, 0.000000, 1.000000),
  RawColor(WHITE, sRGB, 0xFFFFFF, 1.000000, 1.000000, 1.000000, RGB, 1.000000, 1.000000, 1.000000)
"""


class FakeSource:
    """A pinned-OCCT stand-in holding only the two Quantity inputs."""

    def __init__(self, root: Path, enum: str = ENUM, pxx: str = PXX):
        package = root / table.PACKAGE
        package.mkdir(parents=True)
        (package / "Quantity_NameOfColor.hxx").write_text(enum)
        (package / "Quantity_ColorTable.pxx").write_text(pxx)


class GeneratorTest(unittest.TestCase):
    def test_entries_keep_literal_text(self):
        with tempfile.TemporaryDirectory() as directory:
            FakeSource(Path(directory))
            entries = table.table_entries(Path(directory))
        self.assertEqual([name for name, _, _ in entries], ["BLACK", "BLUE", "WHITE"])
        self.assertEqual(entries[1][1], ["0.000000", "0.000000", "1.000000"])

    def test_aliases_are_separated_from_canonical_names(self):
        with tempfile.TemporaryDirectory() as directory:
            FakeSource(Path(directory))
            canonical, aliases = table.enum_members(Path(directory))
        self.assertEqual(canonical, ["BLACK", "BLUE", "WHITE"])
        self.assertEqual(aliases, [("BLUE1", "BLUE")])

    def test_order_mismatch_is_refused(self):
        with tempfile.TemporaryDirectory() as directory:
            FakeSource(Path(directory), enum=ENUM.replace("BLACK,\n  Quantity_NOC_BLUE,", "BLUE,\n  Quantity_NOC_BLACK,"))
            with self.assertRaises(ValueError):
                table.generate(Path(directory))

    def test_unknown_enumerator_shape_is_refused(self):
        with tempfile.TemporaryDirectory() as directory:
            FakeSource(Path(directory), enum=ENUM.replace("Quantity_NOC_WHITE", "Other_WHITE"))
            with self.assertRaises(ValueError):
                table.enum_members(Path(directory))


class CorpusTest(unittest.TestCase):
    def test_bits_round_trip(self):
        self.assertEqual(corpus.f(-1.0), "bf800000")
        self.assertEqual(corpus.d(0.5), "3fe0000000000000")

    def test_names_come_from_the_generated_table(self):
        names = corpus.color_names()
        self.assertEqual((names[0], names[-1], len(names)), ("BLACK", "WHITE", corpus.COLOR_COUNT))

    def test_terminating_native_inputs_are_absent(self):
        lines = set(corpus.cases())
        self.assertNotIn("qc_from_name QUANTITY_NOC_", lines)
        self.assertFalse(any(line.startswith("qc_ctor 2 7ff8") for line in lines))


if __name__ == "__main__":
    unittest.main()
