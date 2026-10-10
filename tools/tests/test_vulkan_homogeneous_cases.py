"""Keep complete homogeneous input coverage in the native comparison matrix."""
from pathlib import Path
import re
import unittest
from vulkan_homogeneous_cases import HOMOGENEOUS_MODES
from verify_vulkan import CASES


class HomogeneousCaptureCasesTests(unittest.TestCase):
    def test_native_matrix_matches_every_rust_fixture(self) -> None:
        root = Path(__file__).resolve().parents[2]
        source = (root / "crates/vulkan/tests/support/homogeneous_arrays.rs").read_text()
        declared = source.split("pub const MODES:", 1)[1].split("];", 1)[0]
        self.assertEqual(tuple(re.findall(r'"(hom-[a-z-]+)"', declared)), HOMOGENEOUS_MODES)
        self.assertEqual(len(HOMOGENEOUS_MODES), len(set(HOMOGENEOUS_MODES)))
        self.assertEqual(len(CASES) * len(HOMOGENEOUS_MODES), 126)
