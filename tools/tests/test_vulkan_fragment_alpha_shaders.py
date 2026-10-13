"""Native state variants cover glyph/nodal/fill interfaces without changing baseline flags."""
import unittest
from pathlib import Path
from vulkan_fragment_alpha_shaders import (compile_fragment_alpha_shaders,
                                           fragment_alpha_table_source, fragment_alpha_variants)


class NamedAlphaCompiler:
    def __init__(self) -> None:
        self.commands: list[list[str]] = []

    def run(self, command: list[str], env: dict[str, str]) -> str:
        self.commands.append(command)
        return ''


class FragmentAlphaShaderTests(unittest.TestCase):
    def test_all_ordinary_fragment_interfaces_have_independent_variants(self) -> None:
        variants = fragment_alpha_variants()
        self.assertEqual(len(variants), 48)
        self.assertEqual(len({stem for stem, _, _ in variants}), 48)
        self.assertIn(('text.clipped', 'text.frag', ('-DMARTENSITE_CLIPPING=1',)), variants)

    def test_compile_uses_vulkan13_and_private_state_define(self) -> None:
        compiler = NamedAlphaCompiler()
        compile_fragment_alpha_shaders(Path('/repo'), Path('/tmp'), {}, compiler.run)
        self.assertEqual(len(compiler.commands), 48)
        self.assertTrue(all('-DMARTENSITE_NATIVE_FRAGMENT_ALPHA=1' in command for command in compiler.commands))
        self.assertTrue(all('vulkan1.3' in command for command in compiler.commands))

    def test_static_catalog_preserves_each_original_shader_identity(self) -> None:
        source = fragment_alpha_table_source()
        self.assertEqual(source.count('native_alpha_pair!'), 48)
        self.assertIn('".native-alpha.frag.spv"', source)
        for stem, _, _ in fragment_alpha_variants():
            self.assertIn(f'native_alpha_pair!("{stem}")', source)


if __name__ == '__main__':
    unittest.main()
