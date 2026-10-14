"""Independent scope requirements for actual retained stencil captures."""

import unittest
from vulkan_ordered_stencil_cases import OrderedStencilCase, ordered_stencil_cases


class OrderedStencilCaseTests(unittest.TestCase):
    def test_every_algorithm_preserves_the_complete_lifecycle_and_layer_family(
        self,
    ) -> None:
        cases = ordered_stencil_cases()
        for mode in (
            "leading-empty",
            "trailing-empty",
            "clear-executed",
            "remove-executed",
            "cap-empty-closed",
            "cap-transparent",
            "layer-clear",
            "layer-shared",
        ):
            self.assertEqual(
                {case.method for case in cases if case.mode == mode and case.seed == 0},
                {"unordered", "weighted", "peeling"},
            )

    def test_mask_controls_distinguish_every_bit_from_the_reference_low_bit(
        self,
    ) -> None:
        seeds = {case.seed for case in ordered_stencil_cases() if case.mode == "enable"}
        self.assertTrue({0, 1, 2}.issubset(seeds))
        self.assertTrue({1 | (1 << bit) for bit in range(1, 8)}.issubset(seeds))

    def test_real_point_line_and_text_rejections_are_not_fill_only_proxy_tests(
        self,
    ) -> None:
        cases = ordered_stencil_cases()
        for mode in ("line", "point", "text"):
            self.assertEqual(
                len([case for case in cases if case.mode == mode and case.seed == 1]), 3
            )

    def test_native_asset_names_are_unique_and_bind_mode_method_and_seed(self) -> None:
        cases = ordered_stencil_cases()
        self.assertEqual(len(cases), 87)
        self.assertEqual(len({case.name for case in cases}), 87)
        self.assertEqual(
            OrderedStencilCase("enable", "weighted", 1).name, "enable-weighted-seed1"
        )
