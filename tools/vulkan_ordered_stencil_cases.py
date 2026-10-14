"""Native-defined ordered stencil cases, including real framebuffer rejection and mask discriminators."""

from __future__ import annotations
from dataclasses import dataclass

STENCIL_MODES: tuple[str, ...] = (
    "enable",
    "disable",
    "toggle",
    "leading-empty",
    "trailing-empty",
    "clear-executed",
    "remove-executed",
    "cap-enable",
    "cap-disable",
    "cap-transparent",
    "cap-empty-closed",
    "layer-shared",
    "layer-clear",
)
STENCIL_METHODS: tuple[str, ...] = ("unordered", "weighted", "peeling")


@dataclass(frozen=True)
class OrderedStencilCase:
    mode: str
    method: str
    seed: int

    @property
    def name(self) -> str:
        """Identify immutable native assets; e.g. `case.name == 'enable-weighted-seed1'`."""
        return f"{self.mode}-{self.method}-seed{self.seed}"


def ordered_stencil_cases() -> tuple[OrderedStencilCase, ...]:
    """Cover ordered groups, lifecycle, caps, layers and all stencil-mask bits; `ordered_stencil_cases()`."""
    modes = tuple((mode, 0) for mode in STENCIL_MODES)
    mask_controls = tuple(("enable", seed) for seed in (1, 2, 3, 5, 9, 17, 33, 65, 129))
    controls = tuple(
        (mode, 1) for mode in ("disable", "toggle", "cap-enable", "cap-transparent")
    )
    annotations = tuple((mode, 1) for mode in ("line", "point", "text"))
    return tuple(
        OrderedStencilCase(mode, method, seed)
        for mode, seed in modes + mask_controls + controls + annotations
        for method in STENCIL_METHODS
    )
