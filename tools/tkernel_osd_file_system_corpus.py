"""Request lines for the OSD file_system probe family (OSD_FileSystem, OSD_LocalFileSystem,
OSD_CachedFileSystem, OSD_FileSystemSelector, OSD_StreamBuffer, OSD_OpenFile).

Covers every libstdc++ openmode combination (app, ate, binary, in, out, trunc) against an
existing and a missing file for buffers, output streams, descriptors and filebufs; input
stream offsets including reuse, -1 and the raising -2; remote/local URL classification;
selector ordering, preference, removal and lazy delegation through logging fakes; the
cached file system's stream and buffer reuse; the default selector; and glibc fopen modes.
"""
from __future__ import annotations

ALL_MODES = range(64)
CONTENTS = ["abcdef", "NONE"]
FOPEN_MODES = ["r", "w", "a", "r+", "w+", "a+", "rb", "wb+", "ab+", "rx", "wx", "w+x", "x",
               "q", "-", "r+b", "rw", "b", "a+x", "rbbbbb+", "w+e", "R"]
URLS = ["file.brep", "/abs/x.step", "http://host/x", "C:/x", "a:b", ":x//y", "content://x",
        "s3://bucket", "x:/y", "-", "ftp:x", "file:///x", "a://"]
SELECTORS = [
    ("a/*/1,b/*/1", "+0,+1", "u"), ("a/*/1,b/*/1", "+0,^1", "u"), ("a/*/1,b/*/1", "+0,+1,+0", "u"),
    ("a/*/0,b/*/1", "+0,+1", "u"), ("a/x:/1,b/*/1", "+0,+1", "x://1"), ("a/x:/1,b/y:/1", "+0,+1", "z://"),
    ("a/*/1,b/*/1", "+0,+1,-0", "u"), ("a/*/1", "+0,-0,-0", "u"), ("a/*/0,b/*/0", "+0,^1", "u"),
    ("a/*/1,b/*/1,c/*/1", "+0,+1,^2,^0,-1", "u"), ("a/*/1", "-", "u"),
]
CACHED = [
    "i1:0,i1:2,i1:-1,i2:0,i1:0", "i1:3,i1:0,b1:1,i1:0", "b1:0,b1:2,b2:1,w1:0,i1:0",
    "i1:0,o1,i1:0,i1:-2,i1:1", "i2:0,i2:10,i2:0", "b1:9,i1:0", "w2:2,i2:0,i2:1",
]


def cases() -> list[str]:
    """Every file_system request line, in a fixed order.

    >>> len([c for c in cases() if c.startswith("osd_fs_default")])
    1
    """
    lines: list[str] = []
    for content in CONTENTS:
        lines += [f"osd_fs_local_buffer {content} {mode} 0 1" for mode in ALL_MODES]
        lines += [f"osd_fs_ostream {content} {mode}" for mode in ALL_MODES]
        lines += [f"osd_fs_fd {content} {mode}" for mode in ALL_MODES]
        lines += [f"osd_fs_filebuf {content} {mode}" for mode in ALL_MODES]
        lines += [f"osd_fs_ctime {content}"]
    for mode in [8, 12, 16, 24, 9, 1, 48, 10]:
        for offset in [2, 6, 10, -1]:
            lines += [f"osd_fs_local_buffer abcdef {mode} {offset} {size}" for size in (0, 1)]
    for content in CONTENTS + ["-"]:
        for mode in [0, 4, 8, 16, 2, 32]:
            for first, second in [(0, -1), (0, 1), (2, 0), (5, -1), (-1, -1), (-2, 0), (0, -2), (9, 1)]:
                lines.append(f"osd_fs_istream {content} {mode} {first} {second}")
    lines += [f"osd_fs_supported {url}" for url in URLS]
    lines += [f"osd_fs_selector {fakes} {ops} {url}" for fakes, ops, url in SELECTORS]
    lines += [f"osd_fs_cached abcdef 123456 {ops}" for ops in CACHED]
    lines += ["osd_fs_cached NONE xyz i1:0,b1:0,i2:0", "osd_fs_default"]
    for content in CONTENTS:
        for index, mode in enumerate(FOPEN_MODES):
            lines.append(f"osd_fs_fopen {content} {mode} {index % 2}")
    for content in CONTENTS:
        for kind in ["i", "o", "io"]:
            for index, mode in enumerate([0, 8, 16, 24, 1, 2, 32, 48, 56, 9, 17, 4, 26]):
                lines.append(f"osd_fs_fstream {content} {kind} {mode} {index % 2}")
    return lines
