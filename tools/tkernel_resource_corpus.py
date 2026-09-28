"""Differential corpus for the Resource probe family (tools/oracle/tkernel_resource_probe.cxx).

Every decoder sees every lead/trail byte pair it distinguishes; every encoder sees every
nonzero UTF-16 unit; the code pages see every byte and every unit up to their largest
entry. Buffer boundaries are exercised except where the native code writes outside the
buffer (undefined behavior the Rust port reports as Standard_OutOfRange instead):
maxsize 0, EUC/GB truncating before anything was written, and UTF-8 into a buffer exactly
the encoded size. Manager cases write their files under target/oracle-tkernel/.
"""
from __future__ import annotations

SJIS, EUC, ANSI, GB, UTF8, LOCALE, GBK, BIG5 = 0, 1, 2, 3, 4, 5, 25, 26
CODE_PAGES = range(6, 25)
DIRECTORY = "target/oracle-tkernel/resource-probe"


def hex_bytes(data: bytes) -> str:
    """Wire spelling of bytes; hex_bytes(b'') == '-'."""
    return data.hex() or "-"


def hex_units(units: list[int]) -> str:
    """Wire spelling of UTF-16 units; hex_units([0x41]) == '0041'."""
    return "".join(f"{unit:04x}" for unit in units) or "-"


def decoder_cases() -> list[str]:
    """Every lead byte of each table decoder against every trail byte."""
    trails = bytes(range(1, 256))
    leads = {SJIS: [*range(0x81, 0xA0), *range(0xE0, 0xF0)], EUC: range(0xA1, 0xFF), GB: range(0x80, 0x100)}
    cases = [f"res_decode {fmt} {hex_bytes(b''.join(bytes([lead, t]) for t in trails))}"
             for fmt, lead_bytes in leads.items() for lead in lead_bytes]
    samples = [b"", b"plain ASCII", trails, b"A\x81", b"\xa1", b"\x80\x7f", b"\x81\x40x\xe0\xfcy"]
    for fmt in [SJIS, EUC, ANSI, GB, UTF8, LOCALE, *CODE_PAGES]:
        cases += [f"res_decode {fmt} {hex_bytes(sample)}" for sample in samples]
    return cases


def utf8_cases() -> list[str]:
    """UTF-8 decoding: valid forms, surrogates, overlongs, illegal and truncated sequences."""
    samples = ["é€😀".encode(), b"\xed\xa0\x80", b"\xc0\xaf", b"\xe0\x80\xaf", b"\xf4\x90\x80\x80",
               b"\xf8\x88\x80\x80\x80", b"\xfc\x84\x80\x80\x80\x80", b"\xff", b"\x80", b"a\xc3",
               b"a\xe2\x82", b"\xc3(", b"abc\xe9def", "日本語".encode()]
    return [f"res_decode {fmt} {hex_bytes(sample)}" for fmt in (UTF8, LOCALE, ANSI) for sample in samples]


def cjk_cases() -> list[str]:
    """GBK and Big5: every lead with every valid trail, invalid trails and four-byte GBK."""
    gbk_trails = bytes([*range(0x40, 0x7F), *range(0x80, 0xFF)])
    big5_trails = bytes([*range(0x40, 0x7F), *range(0xA1, 0xFF)])
    cases = []
    for lead in range(0x81, 0xFF):
        cases.append(f"res_gbk {hex_bytes(b''.join(bytes([lead, t]) for t in gbk_trails))}")
        cases.append(f"res_big5 {hex_bytes(b''.join(bytes([lead, t]) for t in big5_trails))}")
    for trail in [0x01, 0x20, 0x2F, 0x3A, 0x3F, 0x7F, 0x80, 0xA0, 0xFF]:
        for op in ("res_gbk", "res_big5"):
            cases.append(f"{op} {hex_bytes(bytes([0x41, 0x81, trail, 0x42]))}")
    for single in [b"\x80", b"\xff", b"ab\x80cd", b"\x81", b"x\xfe"]:
        cases += [f"res_gbk {hex_bytes(single)}", f"res_big5 {hex_bytes(single)}"]
    for gb1 in (0x81, 0x82, 0x83, 0x84, 0xFE):
        for gb2 in (0x30, 0x35, 0x39):
            for gb3 in (0x81, 0x90, 0xFE):
                sequence = bytes([gb1, gb2, gb3, 0x30, 0x31, 0x39])
                cases.append(f"res_gbk {hex_bytes(b'A' + sequence)}")
                cases.append(f"res_gbk {hex_bytes(sequence + b'Z')}")
    cases += ["res_gbk " + hex_bytes(b"\x81\x30" + bytes([t])) for t in (0x20, 0x41, 0x80, 0xFF)]
    return cases


def table_encoder_cases() -> list[str]:
    """Every nonzero unit through SJIS, EUC, ANSI and GB, 256 units per request."""
    cases = []
    for start in range(0, 0x10000, 256):
        units = [unit for unit in range(start, start + 256) if unit]
        for fmt in (SJIS, EUC, ANSI, GB):
            cases.append(f"res_encode {fmt} {3 * len(units) + 10} {hex_units(units)}")
    return cases


def boundary_cases() -> list[str]:
    """Small buffers around single and double bytes, embedded zeros, and empty text."""
    texts = {"ascii": [0x41, 0x42, 0x43, 0x44], "cjk": [0x4E00, 0x4E8C, 0x4E09], "mixed": [0x41, 0x4E00, 0x42, 0x3042],
             "zero": [0x41, 0, 0x42], "wide": [0x100, 0xFFFF], "empty": []}
    cases = []
    for name, units in texts.items():
        for size in range(1, 10):
            for fmt in (SJIS, EUC, ANSI, GB, LOCALE, GBK, BIG5):
                double_first = bool(units) and units[0] >= 0x3000
                if fmt in (EUC, GB) and double_first and size <= 3:
                    continue  # native writes tostr[-1]
                cases.append(f"res_encode {fmt} {size} {hex_units(units)}")
    return cases


def utf8_length(units: list[int]) -> int:
    """LengthOfCString: UTF-8 bytes of the text up to its first zero, lone surrogates dropped."""
    length, index = 0, 0
    while index < len(units) and units[index]:
        unit = units[index]
        pair = 0xD800 <= unit < 0xDC00 and index + 1 < len(units) and 0xDC00 <= units[index + 1] < 0xE000
        length += 4 if pair else 0 if 0xD800 <= unit < 0xE000 else 1 if unit < 0x80 else 2 if unit < 0x800 else 3
        index += 2 if pair else 1
    return length


def utf8_encoder_cases() -> list[str]:
    """UTF-8 encoding around LengthOfCString, skipping the native one-byte overflow size."""
    texts = [[0x41], [0xE9, 0x20AC], [0xD83D, 0xDE00], [0xD800, 0x41], [0xDC00], [0x41, 0, 0x42], [],
             list(range(0x7E, 0x82)), [0x7FF, 0x800, 0xFFFF]]
    cases = []
    for units in texts:
        length = utf8_length(units)
        for size in {length - 1, length + 1, length + 2, 3 * len(units) + 10} - {length}:
            if size >= 1:
                cases.append(f"res_encode {UTF8} {size} {hex_units(units)}")
    return cases


def code_page_cases() -> list[str]:
    """Every code page: all units up to U+25A0 (its largest entry) and each buffer rule."""
    cases = []
    for fmt in CODE_PAGES:
        for start in range(1, 0x25A1, 128):
            units = list(range(start, min(start + 128, 0x25A1)))
            cases.append(f"res_encode {fmt} {len(units) + 1} {hex_units(units)}")
        for units in ([0x41, 0x20AC, 0x402], [0x41, 0, 0x42], []):
            for size in (len(units) - 1, len(units), len(units) + 1, len(units) + 2):
                if size >= 1:
                    cases.append(f"res_encode {fmt} {size} {hex_units(units)}")
    return cases


RESOURCE_FILES = [
    b"",
    b"key: value\n",
    b"!ignored: yes\n  !key : kept\n bad line\n:invalid\n x : old\nx: new \r\nspace: \\ leading\ntab:\\\tleading\n"
    b"slash:\\\\path\nother:\\n\nempty: \t\nlast: eof",
    b"int: -42extra\nreal: 1.25more\nbad: nothing\nhex: 0x10\nexp: 1e3\nspaces:    7   \n",
    b"long.resource.name.with.dots : a value longer than nine bytes: with colons\n",
    b"a\x00hidden: part\nnext: line\n",
    b"cr: value\r\r\nlonecr: x\ry\n\n   \n\t\t\nk:\n",
    b"\x00starts: with nul\n",
    b"x: 1\n\x00\n",
    b"k1:v1\nk1:v2\n\\: esc\n",
]


def manager_cases() -> list[str]:
    """Loading, verbose/debug traces, both constructors, Save and GetResourcePath."""
    cases = []
    for index, content in enumerate(RESOURCE_FILES):
        for verbose, debug in ((0, 0), (1, 0), (0, 1), (1, 1)):
            cases.append(f"res_load {verbose} {debug} {DIRECTORY}/load{index} {hex_bytes(content)}")
    defaults_file, user_file = hex_bytes(b"a: 1\nb: ref\n"), hex_bytes(b"b: user\nc: 2.5\n")
    for verbose in (0, 1):
        base = f"{DIRECTORY}/dirs{verbose}"
        for defaults, user in [("defaults", "user"), ("defaults/", ""), ("", "user/"), ("", ""), ("missing", "user")]:
            spelled = [f"{base}/{d}".encode() if d else b"" for d in (defaults, user)]
            cases.append(f"res_dirs {verbose} {base} {hex_bytes(spelled[0])} {hex_bytes(spelled[1])} "
                         f"{defaults_file} {user_file}")
    assignments = ["i:" + b"count".hex() + ":-17", "r:" + b"tol".hex() + ":3fb999999999999a",
                   "r:" + b"big".hex() + ":7fe1ccf385ebc8a0", "r:" + b"neg0".hex() + ":8000000000000000",
                   "s:" + b"z".hex() + ":" + b"\\path".hex(), "s:" + b"a".hex() + ":" + b" lead".hex(),
                   "s:" + b"m".hex() + ":" + b"\ttab".hex(), "s:" + b"e".hex() + ":-", "i:" + b"a".hex() + ":5"]
    for verbose in (0, 1):
        cases.append(f"res_save {verbose} {DIRECTORY}/save{verbose} {' '.join(assignments)}")
        cases.append(f"res_save {verbose} {DIRECTORY}/empty{verbose}")
    for directory in ["", "a", "a/", "a/b", "/abs/x", ".", ".hidden", "a/.hidden", "a/b.c", "../up", "a//b",
                      "a/./b", "dir with space/x"]:
        cases += [f"res_path {user} {hex_bytes(directory.encode())}" for user in (0, 1)]
    return cases


def ext_cases() -> list[str]:
    """ExtValue and SetResource(char16_t*) in every format."""
    values = [(b"ascii", [0x41, 0x42]), ("日本".encode("shift_jis"), [0x65E5, 0x672C]), (b"\xc3\xa9", [0xE9]),
              (b"\x81\x40", [0x41, 0x4E00, 0x42])]
    cases = []
    for fmt in [SJIS, EUC, ANSI, GB, UTF8, LOCALE, *CODE_PAGES, GBK, BIG5]:
        cases += [f"res_ext {fmt} {hex_bytes(data)} {hex_units(units)}" for data, units in values]
    return cases


def charset_cases() -> list[str]:
    """ReadFormat: the three recognized names, others and a missing resource select ANSI."""
    files = [b"FormatType: SJIS\n", b"FormatType: EUC\n", b"FormatType: GB\n", b"FormatType: UTF8\n",
             b"FormatType: sjis\n", b"Other: SJIS\n", b"FormatType:GB \n", b""]
    return [f"res_charset {DIRECTORY}/charset{index} {hex_bytes(data)}" for index, data in enumerate(files)]


def cases() -> list[str]:
    """Every Resource request line in a fixed order.

    >>> len(cases()) > 1000
    True
    """
    return (decoder_cases() + utf8_cases() + cjk_cases() + table_encoder_cases() + boundary_cases()
            + utf8_encoder_cases() + code_page_cases() + manager_cases() + ext_cases() + charset_cases())
