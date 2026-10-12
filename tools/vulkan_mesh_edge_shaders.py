"""Compile native same-pass Martensite mesh stages without changing existing shaders."""
from itertools import product
from pathlib import Path
from vulkan_surface_texture_shaders import ShaderRunner, ShaderBinaryStore, FileShaderBinaryStore


def mesh_edge_shader_variants() -> tuple[tuple[str, str, tuple[str, ...]], ...]:
    """Enumerate actual stage interfaces; mesh_edge_shader_variants()."""
    variants: list[tuple[str, str, tuple[str, ...]]] = []
    for lighting, clipped, image in product((False, True), repeat=3):
        names = (('lit', lighting), ('clipped', clipped), ('image', image))
        suffix = ''.join('.' + name for name, enabled in names if enabled)
        flags = tuple('-D' + name + '=1' for name, enabled in
            (('MARTENSITE_LIGHTING', lighting), ('MARTENSITE_CLIPPING', clipped),
             ('MARTENSITE_SURFACE_TEXTURE', image), ('MARTENSITE_MESH_EDGES', True)) if enabled)
        variants.append(('surface.edges' + suffix, 'frag', flags))
        if not image:
            variants.append(('surface.edges' + suffix, 'geom', flags))
    return tuple(variants)


def compile_mesh_edge_shaders(root: Path, folder: Path, env: dict[str, str],
                              runner: ShaderRunner) -> None:
    """Compile every selected native edge stage; compile_mesh_edge_shaders(root, tmp, env, run)."""
    shaders = root / 'crates/vulkan/shaders'
    for stem, stage, flags in mesh_edge_shader_variants():
        source = shaders / ('surface.frag' if stage == 'frag' else 'surface.edges.geom')
        runner(['glslangValidator', *flags, '--target-env', 'vulkan1.3', '-V',
                str(source), '-o', str(folder / f'{stem}.{stage}.spv')], env)


def check_mesh_edge_shader_binaries(root: Path, folder: Path, env: dict[str, str],
                                    runner: ShaderRunner, store: ShaderBinaryStore | None = None) -> None:
    """Reject stale mesh-stage binaries; check_mesh_edge_shader_binaries(root, tmp, env, run)."""
    compile_mesh_edge_shaders(root, folder, env, runner)
    selected = store if store is not None else FileShaderBinaryStore()
    for stem, stage, _ in mesh_edge_shader_variants():
        name = f'{stem}.{stage}.spv'
        if selected.read(folder / name) != selected.read(root / 'crates/vulkan/shaders' / name):
            raise ValueError(f'received stale SPIR-V for {name}; expected current mesh-edge GLSL')
