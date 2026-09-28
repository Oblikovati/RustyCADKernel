"""Differential corpus for the OSD probe families, appended to verify_tkernel's corpus.

Each family module `tkernel_osd_<family>_corpus.py` exposes `cases() -> list[str]`,
one request line per case, answered by the matching native and Rust probe family.
"""
from __future__ import annotations

import importlib

FAMILIES = ["foundation", "path", "files", "file_system", "process", "timing", "threads"]


def cases() -> list[str]:
    """Every OSD request line, family by family in a fixed order.

    >>> isinstance(cases(), list)
    True
    """
    lines: list[str] = []
    for family in FAMILIES:
        lines += importlib.import_module(f"tkernel_osd_{family}_corpus").cases()
    return lines
