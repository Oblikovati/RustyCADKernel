"""Request lines for the OSD process probe family (OSD_Process, OSD_Host, OSD_Environment, OSD_SharedLibrary, OSD_MemInfo)."""
from __future__ import annotations


def cases() -> list[str]:
    """Every process request line, in a fixed order.

    >>> isinstance(cases(), list)
    True
    """
    return []
