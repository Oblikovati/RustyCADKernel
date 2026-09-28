"""Probe encodings shared by the Storage/FSD corpus modules."""
from __future__ import annotations

import struct


# ---------------------------------------------------------------------------- encodings


def hx(data: bytes) -> str:
    """Probe hex, '-' for no bytes."""
    return data.hex() if data else "-"


def ext(text: str) -> str:
    """UTF-16 units as four hex digits each, '-' when empty."""
    units = text.encode("utf-16-be")
    return units.hex() if units else "-"


def ext_units(units: list[int]) -> str:
    return "".join(f"{unit:04x}" for unit in units) or "-"


def bits(value: float) -> str:
    return struct.pack(">d", value).hex()


def fbits(value: float) -> str:
    return struct.pack(">f", value).hex()
