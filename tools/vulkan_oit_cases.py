"""Retained OIT interactions keep sample coverage diagnostics separate from strict parity."""
from itertools import product
from typing import NamedTuple


class OitCaptureCase(NamedTuple):
    """Actual RenderingParams inputs; OitCaptureCase('weighted', 0, 0., 4, False, False, 'two')."""

    method: str
    samples: int
    depth_factor: float
    peeling_layers: int
    reverse: bool
    fixed_function: bool
    fixture: str

    def arguments(self, prefix: str) -> list[str]:
        """Emit identical native/Rust capture arguments; case.arguments('/tmp/oit')."""
        return [self.method, prefix, str(self.samples), str(self.depth_factor),
                str(self.peeling_layers), str(int(self.reverse)),
                str(int(self.fixed_function)), self.fixture]


def oit_interaction_cases() -> tuple[OitCaptureCase, ...]:
    """Cross actual sampling/lighting/clip families; oit_interaction_cases()."""
    fixtures = ('two', 'clipped', 'texture-nearest', 'texture-phong',
                'texture-gouraud', 'texture-facet', 'texture-no-modulate')
    return tuple(OitCaptureCase(method, samples, 0., 4, reverse, fixed, fixture)
                 for fixture, method, samples, reverse, fixed in
                 product(fixtures, ('unordered', 'weighted', 'peeling'),
                         (0, 4), (False, True), (False, True)))


def oit_alpha_fallback_cases() -> tuple[OitCaptureCase, ...]:
    """Keep no-transparent-fill peeling fallback observable; oit_alpha_fallback_cases()."""
    return tuple(OitCaptureCase(method, 4, 0., 4, False, fixed, fixture)
                 for fixture, method, fixed in product(
                     ('opaque', 'mask', 'mask-blend', 'one-mask'),
                     ('unordered', 'weighted', 'peeling'), (False, True)))


def oit_capping_cases() -> tuple[OitCaptureCase, ...]:
    """Exercise genuine cap stencil/material/texture paths; oit_capping_cases()."""
    fixtures = ('cap-blend', 'cap-mask', 'cap-mask-blend', 'cap-object-material',
                'cap-object-texture-blend', 'cap-phong-blend',
                'cap-generated-blend', 'cap-section-edges-blend')
    return tuple(OitCaptureCase(method, samples, 0., 4, False, fixed, fixture)
                 for fixture, method, samples, fixed in product(
                     fixtures, ('unordered', 'weighted', 'peeling'), (0, 4), (False, True)))
