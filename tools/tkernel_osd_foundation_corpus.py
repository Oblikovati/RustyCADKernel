"""Request lines for the OSD foundation probe family (protection, OSD_Error, exceptions).

Covers every Add/Sub protection cell, the default and explicit protections, the Unix
`OSD_Error::Perror` classification of every errno it names (plus unknown and zero errnos)
from every `OSD_WhoAmI` class and two out-of-range classes, and every exception class.
"""
from __future__ import annotations

import errno

# The errnos the Linux Perror switch names, then values it does not name.
NAMED_ERRNOS = ["EBADF", "EINVAL", "EDQUOT", "ENOLCK", "EOPNOTSUPP", "EACCES", "EBUSY",
                "ERANGE", "EPERM", "EROFS", "ENXIO", "EIO", "EISDIR", "EAGAIN", "ENOTDIR",
                "EMLINK", "ELOOP", "EFAULT", "EFBIG", "EINTR", "ENOMEM", "EMFILE",
                "ENAMETOOLONG", "ENFILE", "EXDEV", "ENOENT", "ENOSPC", "ENOTEMPTY", "EEXIST",
                "E2BIG"]
UNNAMED_ERRNOS = [0, -1, 1000, errno.ECHILD, errno.ESRCH, errno.EPIPE]


def cases() -> list[str]:
    """Every foundation request line, in a fixed order.

    >>> len([c for c in cases() if c.startswith("osd_prot_row")])
    16
    """
    lines = [f"osd_prot_row {value}" for value in range(16)]
    lines += ["osd_prot_default", "osd_enums", "osd_prot_set 0 15 7 1", "osd_prot_set 1 11 1 1",
              "osd_prot_set 15 0 8 4"]
    errnos = [getattr(errno, name) for name in NAMED_ERRNOS] + UNNAMED_ERRNOS
    for value in errnos:
        for origin in range(-1, 16):
            lines.append(f"osd_error {value} {origin} ctx")
    for origin in range(-1, 16):
        lines += [f"osd_error {errno.ENOENT} {origin} Open", f"osd_error {errno.ENOENT} {origin} -"]
    lines += [f"osd_exception {index} message_{index}" for index in range(22)]
    lines += ["osd_exception 19 -"]
    return lines
