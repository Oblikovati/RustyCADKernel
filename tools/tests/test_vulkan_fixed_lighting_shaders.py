"""Typed fake compilation verifies all independent fixed-lighting layouts."""
import unittest
from pathlib import Path
from vulkan_fixed_lighting_shaders import (fixed_lighting_shader_variants, fixed_shader_stem,
    compile_fixed_lighting_shaders, check_fixed_lighting_shader_binaries)

class RecordedFixedShaderCompiler:
    def __init__(self) -> None:
        self.commands: list[list[str]] = []
    def __call__(self, command: list[str], env: dict[str, str]) -> str:
        self.commands.append(command)
        return ''

class FixedShaderBinaryStore:
    def __init__(self, stale: bool = False) -> None:
        self.stale = stale
    def read(self, path: Path) -> bytes:
        return b'stale' if self.stale and path.parent == Path('scratch') else b'current'

class FixedLightingShaderTests(unittest.TestCase):
    def test_family_flags_cover_all_thirty_stages(self) -> None:
        variants = fixed_lighting_shader_variants()
        self.assertEqual(len(variants), 30)
        self.assertEqual(len({(stem, stage) for stem, stage, _ in variants}), 30)
        self.assertTrue(all(any('FIXED' in flag for flag in flags) for _, _, flags in variants))
        self.assertEqual(fixed_shader_stem('lighting', True, 'line')[0], 'lighting.fixed.line.clipped')
        self.assertEqual(fixed_shader_stem('surface', True, 'image')[0], 'surface.fixed.clipped.image')

    def test_source_selection_and_current_binary_gate(self) -> None:
        compiler = RecordedFixedShaderCompiler()
        compile_fixed_lighting_shaders(Path('root'), Path('scratch'), {}, compiler)
        self.assertEqual(len(compiler.commands), 30)
        self.assertTrue(any('root/crates/vulkan/shaders/surface.edges.geom' in row for row in compiler.commands))
        check_fixed_lighting_shader_binaries(Path('root'), Path('scratch'), {}, compiler, FixedShaderBinaryStore())
        with self.assertRaisesRegex(ValueError, 'stale fixed SPIR-V'):
            check_fixed_lighting_shader_binaries(Path('root'), Path('scratch'), {}, compiler, FixedShaderBinaryStore(True))
