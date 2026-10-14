"""Check observed native storage without inventing a drawable internal format; e.g. validate_attachment(row)."""

from __future__ import annotations


def validate_attachment(row: dict[str, int]) -> None:
    """Validate actual queried storage; e.g. validate_attachment(trace_record)."""
    kind = row["attachment_kind"]
    expected = _attachment_expected(kind)
    actual = tuple(row[name] for name in _STENCIL_STORAGE_FIELDS)
    if actual != expected:
        raise ValueError(
            f"attachment kind={kind} storage={actual}; expected {expected}"
        )
    if kind == 33304 and row["framebuffer"] != 0:
        raise ValueError(
            f"default attachment framebuffer={row['framebuffer']}; expected 0"
        )


_STENCIL_STORAGE_FIELDS = (
    "attachment_internal_format",
    "attachment_stencil_size",
    "attachment_component_type",
)


def _attachment_expected(kind: int) -> tuple[int, int, int]:
    """Keep unavailable/default distinct from known storage; e.g. _attachment_expected(33304)."""
    if kind == 0:
        return (-1, -1, -1)
    if kind == 33304:
        return (-1, 8, 35863)
    if kind in (5890, 36161):
        return (35056, 8, 35863)
    raise ValueError(
        f"attachment kind={kind}; expected NONE, default drawable, texture or renderbuffer"
    )
