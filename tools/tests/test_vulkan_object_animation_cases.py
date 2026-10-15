"""Object capture families and exact native animation requests stay in full verification."""

import unittest
from verify_vulkan import capture_modes, parse_arguments
from vulkan_object_animation_cases import OBJECT_ANIMATION_MODES
from tkv3d_corpus_animation_object import append_object_requests
from verify_animation_object import corpus


class ObjectAnimationCaseTests(unittest.TestCase):
    def test_capture_modes_are_selected_and_join_full_corpus(self) -> None:
        self.assertEqual(capture_modes("object-animation"), OBJECT_ANIMATION_MODES)
        self.assertEqual(len(OBJECT_ANIMATION_MODES), 10)
        self.assertTrue(set(OBJECT_ANIMATION_MODES).issubset(capture_modes("all")))
        options = parse_arguments(
            [
                "--source",
                "OCCT",
                "--build",
                "build",
                "--output",
                "result.json",
                "--case-family",
                "object-animation",
            ]
        )
        self.assertEqual(options.case_family, "object-animation")

    def test_numeric_corpus_covers_nulls_and_immediate_layer_policy(self) -> None:
        rows: list[str] = []
        append_object_requests(rows)
        self.assertEqual(len(rows), 2488)
        self.assertEqual(len(set(rows)), 2488)
        self.assertIn("v3_animation_object 2 3 2.0 0.0 1.0", rows)
        self.assertIn("v3_base_animation_object 216 1", rows)
        self.assertEqual(corpus(), "\n".join(rows) + "\n")


if __name__ == "__main__":
    unittest.main()
