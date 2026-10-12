"""Build independent Martensite fixed-lighting binaries without changing Phong SPIR-V."""
from itertools import product
from pathlib import Path
from vulkan_surface_texture_shaders import ShaderRunner, ShaderBinaryStore, FileShaderBinaryStore

ShaderVariant = tuple[str, str, tuple[str, ...]]

def fixed_lighting_shader_variants() -> tuple[ShaderVariant, ...]:
    """Enumerate every fixed interface; fixed_lighting_shader_variants()."""
    variants: list[ShaderVariant] = []
    for family, clipped in product(('lighting', 'surface'), (False, True)):
        extras = ('', 'line', 'point') if family == 'lighting' else ('', 'image')
        for extra in extras:
            stem, flags = fixed_shader_stem(family, clipped, extra)
            variants.extend((stem, stage, flags) for stage in ('vert', 'frag'))
    for clipped, image in product((False, True), repeat=2):
        stem, flags = fixed_shader_stem('surface.edges', clipped, 'image' if image else '')
        variants.append((stem, 'frag', flags))
        if not image:
            variants.append((stem, 'geom', flags))
    for clipped in (False,True):
        stem = 'line.fixed' + ('.clipped' if clipped else '')
        flags = ('-DMARTENSITE_FIXED_LINE_STIPPLE=1',) + (('-DMARTENSITE_CLIPPING=1',) if clipped else ())
        variants.extend((stem,stage,flags) for stage in ('vert','frag'))
    return tuple(variants)

def fixed_shader_stem(family: str, clipped: bool, extra: str) -> tuple[str, tuple[str, ...]]:
    """Map one native interface; fixed_shader_stem('surface', True, 'image')."""
    names = (('fixed', True), (extra, family == 'lighting' and bool(extra)),
             ('clipped', clipped), ('image', family != 'lighting' and extra == 'image'))
    stem = family + ''.join('.' + name for name, enabled in names if enabled)
    flags = (('MARTENSITE_FIXED_LIGHTING', True), ('MARTENSITE_LIGHTING', family != 'lighting'),
             ('MARTENSITE_CLIPPING', clipped), ('MARTENSITE_LINE_LIGHTING', extra == 'line'),
             ('MARTENSITE_MARKER_LIGHTING', extra == 'point'),
             ('MARTENSITE_SURFACE_TEXTURE', extra == 'image'),
             ('MARTENSITE_MESH_EDGES', family == 'surface.edges'))
    return stem, tuple('-D' + name + '=1' for name, enabled in flags if enabled)

def compile_fixed_lighting_shaders(root: Path, folder: Path, env: dict[str, str], runner: ShaderRunner) -> None:
    """Compile genuine shader interfaces; compile_fixed_lighting_shaders(root, tmp, env, run)."""
    shaders = root / 'crates/vulkan/shaders'
    for stem, stage, flags in fixed_lighting_shader_variants():
        family = 'lighting' if stem.startswith('lighting.') else 'line' if stem.startswith('line.') else 'surface'
        source = shaders / (f'{family}.{stage}' if stage != 'geom' else 'surface.edges.geom')
        runner(['glslangValidator', *flags, '--target-env', 'vulkan1.3', '-V',
                str(source), '-o', str(folder / f'{stem}.{stage}.spv')], env)

def check_fixed_lighting_shader_binaries(root: Path, folder: Path, env: dict[str, str],
                                        runner: ShaderRunner, store: ShaderBinaryStore | None = None) -> None:
    """Require current fixed binaries; check_fixed_lighting_shader_binaries(root, tmp, env, run)."""
    compile_fixed_lighting_shaders(root, folder, env, runner)
    selected = store if store is not None else FileShaderBinaryStore()
    for stem, stage, _ in fixed_lighting_shader_variants():
        name = f'{stem}.{stage}.spv'
        if selected.read(folder / name) != selected.read(root / 'crates/vulkan/shaders' / name):
            raise ValueError(f'received stale fixed SPIR-V for {name}; expected current fixed lighting GLSL')
