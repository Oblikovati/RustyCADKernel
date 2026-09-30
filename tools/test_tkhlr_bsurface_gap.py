"""Strict missing-symbol evidence validation; python3 -m unittest discover -s tools -p test_tkhlr_bsurface_gap.py."""
import unittest
from tkhlr_bsurface_gap import validate_axis_failure

class AxisGapValidationTests(unittest.TestCase):
    """A declaration gap must never hide unrelated compiler/linker failures."""
    def test_accepts_only_expected_symbol(self) -> None:
        """The actual native diagnostic is accepted."""
        diagnostic="/usr/bin/ld: x.o: undefined reference to `HLRBRep_BSurfaceTool::Axis(BRepAdaptor_Surface const&)'\ncollect2: error: ld returned 1 exit status\n"
        validate_axis_failure(1,diagnostic)

    def test_rejects_success_or_unrelated_error(self) -> None:
        """A newly implemented member or changed failure invalidates the gap."""
        for code,diagnostic in [(0,''),(1,"undefined reference to `Other::Function()'"),(1,'fatal error: file not found')]:
            with self.subTest(code=code,diagnostic=diagnostic):
                with self.assertRaises(RuntimeError):
                    validate_axis_failure(code,diagnostic)

    def test_rejects_extra_diagnostic(self) -> None:
        """The expected symbol cannot mask an additional failure."""
        diagnostic="undefined reference to `HLRBRep_BSurfaceTool::Axis(BRepAdaptor_Surface const&)'\nfatal error: unrelated\n"
        with self.assertRaises(RuntimeError):
            validate_axis_failure(1,diagnostic)
