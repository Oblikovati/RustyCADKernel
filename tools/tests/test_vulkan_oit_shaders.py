"""Named compiler observations preserve OIT fill-only and multisample interfaces."""
from pathlib import Path
import unittest
from vulkan_oit_shaders import (compile_oit_shaders, oit_fill_shader_variants,
                                oit_geometry_shader_variants, oit_composite_shader_variants,
                                oit_shader_table_source)


class NamedOitCompiler:
    def __init__(self) -> None:
        self.commands: list[list[str]] = []

    def run(self, command: list[str], environment: dict[str, str]) -> str:
        self.commands.append(command)
        return ""


class OitShaderTests(unittest.TestCase):
    def test_static_table_packages_every_compiled_fragment_without_runtime_io(self) -> None:
        source = oit_shader_table_source()
        for stem, _, _ in oit_geometry_shader_variants():
            self.assertIn(f'oit_fragment!("{stem}")', source)
        self.assertEqual(source.count("oit_fragment!"), len(oit_geometry_shader_variants()))
        self.assertIn('include_bytes!(concat!("../../shaders/", $stem, ".frag.spv"))', source)

    def test_native_filter_keeps_nodal_and_glyph_shaders_outside_oit(self) -> None:
        variants = oit_fill_shader_variants()
        self.assertEqual(len({stem for stem, _, _ in variants}), len(variants))
        self.assertTrue(any(stem == "surface.edges.fixed.clipped.image" for stem, _, _ in variants))
        self.assertFalse(any(part in stem.split(".") for stem, _, _ in variants
                             for part in ("line", "point", "text")))

    def test_initial_peeling_and_fixed_output_are_distinct_compilations(self) -> None:
        variants = oit_geometry_shader_variants()
        self.assertEqual(len(variants), len(oit_fill_shader_variants()) * 6)
        self.assertEqual(len({stem for stem, _, _ in variants}), len(variants))
        initial = next(flags for stem, _, flags in variants if stem == "unlit.oit.initial.fixed-output")
        self.assertIn("-DMARTENSITE_OIT_INITIAL=1", initial)
        self.assertIn("-DMARTENSITE_OIT_FIXED_OUTPUT=1", initial)

    def test_compositors_and_compiler_use_vulkan13_with_sample_id_variant(self) -> None:
        compiler = NamedOitCompiler()
        compile_oit_shaders(Path("/repo"), Path("/tmp/oit"), {}, compiler.run)
        self.assertEqual(len(compiler.commands), len(oit_geometry_shader_variants()) + 5)
        self.assertTrue(all("vulkan1.3" in command for command in compiler.commands))
        self.assertIn("-DMARTENSITE_OIT_MSAA=1", oit_composite_shader_variants()[2][2])


if __name__ == "__main__":
    unittest.main()
