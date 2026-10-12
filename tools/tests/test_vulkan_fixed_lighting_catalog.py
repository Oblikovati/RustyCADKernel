"""Pure source-generation regressions for the pinned lighting fixture catalog."""
import unittest
from vulkan_fixed_lighting_catalog import render_asset_catalog


class FixedLightingCatalogTests(unittest.TestCase):
    def test_profile_order_and_texture_policy_are_explicit(self) -> None:
        source = render_asset_catalog(["lit-basic"], False)
        self.assertIn('"lit-basic",\n        false,', source)
        self.assertLess(source.index("host/lit-basic.depth"), source.index("software/lit-basic.rgba"))
        self.assertEqual(source.count("include_bytes!"), 4)
        self.assertIn("        true,", render_asset_catalog(["lit-nearest"], True))

    def test_invalid_fixture_name_has_offending_value_and_expected_shape(self) -> None:
        with self.assertRaisesRegex(ValueError, "received mode='bad/asset'; expected lowercase"):
            render_asset_catalog(["bad/asset"], False)

    def test_empty_catalog_is_valid_and_repeatable(self) -> None:
        self.assertEqual(render_asset_catalog([], False), render_asset_catalog([], False))
        self.assertTrue(render_asset_catalog([], False).endswith("];\n"))
