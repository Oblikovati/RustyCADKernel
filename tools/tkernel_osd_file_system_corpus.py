"""Request lines for the OSD file_system probe family (OSD_FileSystem, OSD_LocalFileSystem, OSD_CachedFileSystem, OSD_FileSystemSelector, OSD_StreamBuffer, OSD_OpenFile)."""
from __future__ import annotations


def cases() -> list[str]:
    """Every file_system request line, in a fixed order.

    >>> isinstance(cases(), list)
    True
    """
    return []
