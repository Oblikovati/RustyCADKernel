"""Independent mesh stages preserve every surface interface and reject stale SPIR-V."""
import unittest
from pathlib import Path
from test_vulkan_surface_texture_shaders import FakeShaderCompiler, FakeShaderBinaryStore
from vulkan_mesh_edge_shaders import (mesh_edge_shader_variants, compile_mesh_edge_shaders,
                                      check_mesh_edge_shader_binaries)


class MeshEdgeShaderTests(unittest.TestCase):
    def test_mesh_shader_interfaces_cover_lighting_clipping_and_images(self) -> None:
        variants = mesh_edge_shader_variants()
        self.assertEqual(len(variants), 12)
        self.assertEqual(sum(stage == 'geom' for _, stage, _ in variants), 4)
        self.assertEqual(sum(stage == 'frag' for _, stage, _ in variants), 8)
        self.assertTrue(all('-DMARTENSITE_MESH_EDGES=1' in flags for _, _, flags in variants))

    def test_compiler_uses_real_geometry_and_shared_surface_fragment_source(self) -> None:
        compiler = FakeShaderCompiler()
        compile_mesh_edge_shaders(Path('/repo'), Path('/tmp/shaders'), {}, compiler.run)
        self.assertEqual(len(compiler.commands), 12)
        self.assertTrue(all('vulkan1.3' in command for command in compiler.commands))
        self.assertEqual(sum('surface.edges.geom' in ' '.join(command) for command in compiler.commands), 4)

    def test_matching_binaries_pass_and_stale_geometry_is_rejected(self) -> None:
        compiler = FakeShaderCompiler()
        check_mesh_edge_shader_binaries(Path('/repo'), Path('/tmp/shaders'), {}, compiler.run, FakeShaderBinaryStore())
        with self.assertRaisesRegex(ValueError, 'stale SPIR-V.*surface.edges.frag.spv'):
            check_mesh_edge_shader_binaries(Path('/repo'), Path('/tmp/shaders'), {}, compiler.run, FakeShaderBinaryStore(True))


if __name__ == '__main__':
    unittest.main()
