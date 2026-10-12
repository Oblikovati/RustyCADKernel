"""Compile Martensite sampled-image shaders independently from glyph/point shader interfaces."""
from __future__ import annotations
from pathlib import Path
from collections.abc import Callable
from typing import Protocol
from itertools import product
ShaderRunner = Callable[[list[str], dict[str, str]], str]
class ShaderBinaryStore(Protocol):
    def read(self, path: Path) -> bytes: ...
class FileShaderBinaryStore:
    def read(self, path: Path) -> bytes:
        """Read one complete binary; store.read(path)."""
        return path.read_bytes()
def surface_shader_variants() -> tuple[tuple[str, tuple[str, ...]], ...]:
    """Enumerate distinct sampled layouts; surface_shader_variants()."""
    variants: list[tuple[str, tuple[str, ...]]] = []
    for lighting, clipped, image in product((False, True), repeat=3):
        names = (('lit', lighting), ('clipped', clipped), ('image', image))
        stem = 'surface' + ''.join('.' + name for name, enabled in names if enabled)
        flags = tuple('-D' + name + '=1' for name, enabled in
            (('MARTENSITE_LIGHTING', lighting), ('MARTENSITE_CLIPPING', clipped),
             ('MARTENSITE_SURFACE_TEXTURE', image)) if enabled)
        variants.append((stem, flags))
    return tuple(variants)
def check_surface_shader_binaries(root: Path, folder: Path, env: dict[str, str],
                                  runner: ShaderRunner, store: ShaderBinaryStore | None = None) -> None:
    """Require every shader to match source; check_surface_shader_binaries(root, tmp, env, run)."""
    selected = store if store is not None else FileShaderBinaryStore()
    shaders = root / 'crates/vulkan/shaders'
    for stem, flags in surface_shader_variants():
        for stage in ('vert', 'frag'):
            output = folder / f'{stem}.{stage}.spv'
            runner(['glslangValidator', *flags, '--target-env', 'vulkan1.3', '-V',
                    str(shaders / f'surface.{stage}'), '-o', str(output)], env)
            if selected.read(output) != selected.read(shaders / output.name):
                raise ValueError(f'received stale SPIR-V for {output.name}; expected current surface GLSL')
