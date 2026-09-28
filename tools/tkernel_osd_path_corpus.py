"""Request lines for the OSD path probe family (OSD_Path).

Covers parsing in every OSD_SysType (Unix, DOS, VMS and MacOS dialects, and the empty
arms), component round trips through the seven-value constructor and the setters, every
trek edit at valid and out-of-range positions, RelativePath/AbsolutePath on Unix and
drive-letter spellings, the name/extension splitters and the static syntax predicates,
plus seeded random paths over the separator alphabet.
"""
from __future__ import annotations

import random

SYS_TYPES = range(12)

PATHS = [
    "", "/", "//", "a", "/tmp/part.step", "/tmp/dir/", "tmp/dir/file", "../a/b.c", "./a",
    "a/../b/./c.d.e", "/a//b///c", "node:/home/user/f.txt", "host:/", ":/x", "a:b:c",
    "a::b:cde", "::a", "a:", "C:\\one\\two\\file.ext", "C:one\\two", "\\\\Server\\Share\\f.x",
    "C:\\..\\x\\..\\y", ".bashrc", "file.", "archive.tar.gz", "name.averyveryverylongextension",
    "node\"user pass\"::disk:[dir.sub]name.ext", "node::disk:[000000.dir]f.c",
    "disk:[.rel.-]a.b.c", "[dir]file", "disk:[]x", "\"user", "\"user pass\"::d:[a]b",
    "\"u\"", "node::", "disk:name.e1.e2", ".a", "a.", "..", "...", "....x", "a|b", "a^b",
    "http://host/path/f.step", "content://provider/x", "\\\\?\\C:\\x", "\\\\?\\UNC\\s\\x",
    "$DISK:[x]y", "12345678901234567890123456789012", "a:12345678901234567890123456789012",
    "a*b", "a?b", "a<b>c", "a\"b", "x@y", "x\\y", "C:/Program Files/app.exe",
]

COMPONENTS = [
    ["", "", "", "", "", "", ""],
    ["", "", "", "", "|tmp|", "part", ".step"],
    ["host", "", "", "", "|home|", "f", ".txt"],
    ["host", "user", "", "", "|a|^|b|", "n", ".e"],
    ["host", "user", "pw", "DISK$1", "a|b|", "n", ""],
    ["", "user", "pw", "C:", "|x|y|", "n", "e"],
    ["", "", "", "C", "|x|", "n", ".e"],
    ["", "", "", "c:", "", "n", ".e"],
    ["", "", "", "DATA$", "", "n", ".e"],
    ["", "", "", "DATA", "|a", "n", ".e"],
    ["n", "u", "p", "d", "^|^|x", "name", ".ext"],
    ["", "", "", "", "|", "", ""],
    ["", "", "", "", "^", "", ".e"],
    ["", "", "", "", "||srv|share|", "f", ".x"],
]

TREKS = ["", "|", "|a", "|a|", "|a|b|c|", "a|b|", "|ab|b|", "|b|ab|", "||srv|share|", "^|^|x|",
         "|a||b|", "|x|y|x|"]
TREK_OPS = ["up", "down 737562", "down 7c", "down 7c737562", "down 7375627c",
            "rm 0", "rm 1", "rm 2", "rm 3", "rm 4", "rm -1",
            "rmname 62", "rmname 61", "rmname 7c61", "rmname 78", "rmname 616263",
            "value 0", "value 1", "value 2", "value 3", "value 5",
            "insert 6e6577 0", "insert 6e6577 1", "insert 6e6577 2", "insert 6e6577 3",
            "insert 6e65777c 1", "insert 7c6e6577 1", "insert 6e6577 9"]

RELATIVE = [
    ("/a/b", "/a/c/f"), ("/a/b", "/a/b/f"), ("/a/b/", "/a/b//c/f"), ("/a", "/a"), ("/a", "/a/"),
    ("/a/b", "/x/y"), ("/", "/f"), ("/a", "/"), ("a", "/a/f"), ("/a", "a/f"), ("", "/a"),
    ("/a/b/c", "/a/f"), ("/A/b", "/a/b/f"), ("C:\\a\\b", "C:\\a\\c\\f"), ("C:\\a", "c:\\A\\f"),
    ("C:\\a", "D:\\a\\f"), ("C:\\a", "C:a\\f"), ("C:/a/b", "C:/a/b"), ("/a/b", "/a/bb/f"),
    ("/a//b", "/a/b/f"), ("/a/b", "/a"),
]
ABSOLUTE = [
    ("/a/b", "../c"), ("/a/b", "c/d"), ("/a/b", "/abs"), ("/a/b", "C:\\x"), ("rel", "x"),
    ("/a/b", "../../../x"), ("/a/b", "../"), ("/a/b", "..//x"), ("/a/b/", ".\\x\\y"),
    ("C:\\a\\b", "..\\c"), ("/", "x"), ("//srv/a", "b"), ("/a", ""), ("/a/b", "../../x"),
    ("/a/b", "..."), ("/a//b//", "c//"),
]
SPLITS = ["", "a", "a/", "/a", "a\\b.c", "dir/file.STEP", ".hidden", "x.a", "a.b.c",
          "name.ABCDEFGHIJKLMNOPQRS", "name.ABCDEFGHIJKLMNOPQRST", "a/b/.c", "f.", "..", "a.B"]
KINDS = ["", "/", "//", "/a", "C:", "C:x", "x", "\\\\?\\a", "\\\\?\\UNC\\a", "\\\\s\\x", "\\\\?x",
         "http://a", "://a", "a:b://c", "content://x", "content:/x", "c:/x", "a:/"]


def hexed(text: str) -> str:
    """Lowercase hex of the UTF-8 bytes, "-" for empty.

    >>> hexed("a/")
    '612f'
    """
    return text.encode().hex() if text else "-"


def random_path(rng: random.Random) -> str:
    """A short path over the separator-rich alphabet the parsers branch on.

    >>> len(random_path(random.Random(1))) <= 12
    True
    """
    return "".join(rng.choice("ab./\\:|^[]\"-$") for _ in range(rng.randrange(13)))


def mac_reads_past_end(path: str) -> bool:
    """True when MacExtract's `&p[pos + 1]` would read past the terminator (a trailing bar).

    >>> mac_reads_past_end("a:"), mac_reads_past_end("a::"), mac_reads_past_end("a|")
    (True, False, True)
    """
    while "::" in path:
        path = path.replace("::", "^", 1)
    return path.replace(":", "|").endswith("|")


def parse_line(kind: int, path: str) -> list[str]:
    """The parse request, omitted where the native parser reads indeterminate memory.

    >>> parse_line(7, "a:"), parse_line(4, "a:")
    ([], ['osd_path_parse 4 613a'])
    """
    if kind == 7 and mac_reads_past_end(path):
        return []
    return [f"osd_path_parse {kind} {hexed(path)}"]


def cases() -> list[str]:
    """Every path request line, in a fixed order.

    >>> any(line.startswith("osd_path_trek") for line in cases())
    True
    """
    lines = [line for path in PATHS for kind in SYS_TYPES for line in parse_line(kind, path)]
    for values in COMPONENTS:
        encoded = " ".join(hexed(value) for value in values)
        lines += [f"osd_path_values {encoded}", f"osd_path_setters {encoded}"]
    lines += [f"osd_path_trek {hexed(trek)} {op}" for trek in TREKS for op in TREK_OPS]
    lines += [f"osd_path_rel {hexed(a)} {hexed(b)}" for a, b in RELATIVE]
    lines += [f"osd_path_abs {hexed(a)} {hexed(b)}" for a, b in ABSOLUTE]
    lines += [f"osd_path_split {hexed(path)}" for path in SPLITS + PATHS]
    lines += [f"osd_path_kind {hexed(path)}" for path in KINDS + PATHS]
    rng = random.Random(2002)
    for _ in range(200):
        path = random_path(rng)
        lines += parse_line(rng.choice([1, 2, 4, 5, 7, 9]), path)
        lines.append(f"osd_path_split {hexed(path)}")
    return lines
