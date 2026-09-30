"""A missing upstream definition cannot silently become accepted executable coverage."""
import unittest
from tkbo_upstream_gaps import validate_isvalid_failure

EXPECTED_DIAGNOSTIC = (
    "/usr/bin/ld: probe.o: in function `main':\n"
    "probe.cxx:(.text+0x30): undefined reference to `IntTools_PntOnFace::IsValid() const'\n"
    "collect2: error: ld returned 1 exit status\n"
)


class MissingIsValidDiagnosticsTests(unittest.TestCase):
    def test_only_expected_missing_symbol_is_accepted(self) -> None:
        validate_isvalid_failure(1, EXPECTED_DIAGNOSTIC)
        with self.assertRaises(RuntimeError):
            validate_isvalid_failure(0, EXPECTED_DIAGNOSTIC)

    def test_successful_link_or_other_symbol_rejects_gap_evidence(self) -> None:
        for code, text in [(0, ''), (1, ''), (1, EXPECTED_DIAGNOSTIC.replace('IsValid', 'Valid'))]:
            with self.subTest(code=code, diagnostic=text):
                with self.assertRaises(RuntimeError):
                    validate_isvalid_failure(code, text)

    def test_additional_linker_failure_cannot_be_hidden(self) -> None:
        for extra in ["ld: cannot find -lTKBO\n", "ld: undefined reference to `Other()'\n"]:
            with self.subTest(extra=extra):
                with self.assertRaises(RuntimeError):
                    validate_isvalid_failure(1, EXPECTED_DIAGNOSTIC+extra)
