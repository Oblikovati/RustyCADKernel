"""Named fake compiler/storage verifies complete surface variants and strict stale-binary rejection."""
import unittest
from pathlib import Path
from vulkan_surface_texture_shaders import check_surface_shader_binaries, surface_shader_variants
class FakeShaderCompiler:
    def __init__(self) -> None:
        self.commands: list[list[str]] = []
    def run(self, command: list[str], environment: dict[str, str]) -> str:
        self.commands.append(command)
        return ''
class FakeShaderBinaryStore:
    def __init__(self, stale: bool = False) -> None:
        self.stale = stale
    def read(self, path: Path) -> bytes:
        return b'stale' if self.stale and path.parent == Path('/tmp/shaders') else path.name.encode()
class SurfaceShaderTests(unittest.TestCase):
    def test_distinct_layout_variants_are_complete_and_macro_names_match(self) -> None:
        variants = surface_shader_variants()
        self.assertEqual(len(variants), 8)
        self.assertEqual(len({stem for stem, _ in variants}), 8)
        self.assertEqual(variants[-1], ('surface.lit.clipped.image',
            ('-DMARTENSITE_LIGHTING=1', '-DMARTENSITE_CLIPPING=1', '-DMARTENSITE_SURFACE_TEXTURE=1')))
    def test_both_stages_compile_against_vulkan13_with_owned_io_seams(self) -> None:
        compiler = FakeShaderCompiler()
        check_surface_shader_binaries(Path('/repo'), Path('/tmp/shaders'), {}, compiler.run, FakeShaderBinaryStore())
        self.assertEqual(len(compiler.commands), 16)
        self.assertTrue(all('vulkan1.3' in command for command in compiler.commands))
    def test_stale_surface_binary_is_rejected(self) -> None:
        compiler = FakeShaderCompiler()
        with self.assertRaisesRegex(ValueError, 'stale SPIR-V.*surface.vert.spv'):
            check_surface_shader_binaries(Path('/repo'), Path('/tmp/shaders'), {}, compiler.run, FakeShaderBinaryStore(True))
if __name__ == '__main__':
    unittest.main()
