"""Requests for Standard_Dump (operation prefix `services_dump_`).

Excluded inputs (native undefined behavior, never probed):
- InitRealValues with a position of INT_MAX (position + 1 overflows a signed int);
- a variadic count that differs from the number of pointers passed (the probe passes
  exactly `count` outputs, 0 to 4);
- stream positions past the written text (seekp fails and the stream stops writing).
"""
import random
import struct

INT_MIN, INT_MAX = -(2**31), 2**31 - 1
DUMPS = [
    b'"className": "gp_Ax2", "Location": [1, 2, 3], "Direction": [0, 0, 1]',
    b'{"className": "gp_Pnt", "gp_XYZ": [1, 2, 3]}',
    b'"Transient": {"className": "TopLoc_Datum3D", "Trsf": {"Form": 0}}, "Power": 1',
    b'"a": "", "b": "x", "c": {}, "d": [], "a": 1, "a": 2, "a_1": 3',
    b'"key": value, "n": -1.5e-3}', b'"key": {"inner": [1, 2]}, "tail": "t"', b'"k": 1',
    b'"k": ', b'"key', b'', b', ', b',', b':', b'x', b'"k"1', b'"k", 1', b'"k": "unterminated',
    b'{"a": 1}', b'"a": [1, [2, 3], 4], "b": {"c": {"d": 1}}', b'"q": "a, b", "r": "}"',
    b'"nul": "a\x00b", "u": "\xc3\xa9"', b'"x": 1, ', b'"x":1', b'"": 1', b'"s": "v"',
]
FORMATS = [
    b'', b'{', b'}', b'x', b'"a": 1', b'{"a": 1}', b'"a": 1, "b": [1, 2, 3], "c": {"d": 2}',
    b'"a": 1,', b'"a": 1, ', b'"a": 1,, "b"', b'line\nfeed, x', b'[1, 2], [3]', b'}}{{', b'a\x00b, c',
    b'{"className": "Bnd_Box", "CornerMin": [0, 0, 0], "CornerMax": [1, 1, 1], "Gap": 1e-07, "Flags": 0}',
]
FIELDS = [
    b'myValue', b'&myValue', b'aValue', b'anObject', b'angle', b'area', b'a', b'an', b'ab', b'my', b'myX',
    b'My', b'MyX', b'mYX', b'myX.get()', b'aValue.get()', b'Foo()', b'()', b'.get()', b'x()', b'&', b'',
    b'&&a', b'my\x00x', b'\xc3\xa9t\xc3\xa9', b'&anA()',
]
REALS = [1.0, 0.5, -0.0, 0.0, 1e-5, 123456789.0, 1e300, 5e-324, float("inf"), float("-inf"), float("nan"), 1 / 3]


def dump_requests(rng: random.Random) -> list[str]:
    """Every Standard_Dump request; dump_requests(random.Random(1))."""
    return (writer_requests() + format_requests() + split_requests(rng) + reader_requests(rng)
            + [f"services_dump_field_to_name {hex_of(field)}" for field in FIELDS] + ["services_dump_keys"])


def hex_of(data: bytes) -> str:
    """Hex field as the probes decode it, "-" for no bytes; hex_of(b"A") == "41"."""
    return data.hex() if data else "-"


def bits(value: float) -> int:
    """IEEE-754 bit pattern of a double; bits(1.0) == 0x3FF0000000000000."""
    return struct.unpack("<Q", struct.pack("<d", value))[0]


def writer_requests() -> list[str]:
    """Separators, keyed classes, value lists and pointer text."""
    requests = []
    for mode in "sox":
        for prefix in [b"", b"{", b"a", b"a, ", b"{\x00x", b"abc", b"x,", b"x, y"]:
            requests += [f"services_dump_separator {mode} {hex_of(prefix)} {pos}"
                         for pos in sorted({0, 1, len(prefix)}) if pos <= len(prefix)]
        requests += [f"services_dump_key_to_class {mode} {hex_of(prefix)} {hex_of(key)} {hex_of(field)}"
                     for prefix, key, field in [(b"", b"k", b"v"), (b"{", b"Loc", b'"x": 1'), (b"a", b"", b""),
                                                (b"a, ", b"k\x00z", b"f\x00g")]]
    values = [[], [b""], [b"a"], [b"a", b"b\x00c"], [b"x", b"", b"\xc3\xa9"], [b"1", b"2", b"3", b"4"]]
    requests += ["services_dump_character_values " + " ".join(hex_of(v) for v in group) for group in values]
    requests += ["services_dump_real_values " + " ".join(str(bits(v)) for v in REALS[i:i + 4])
                 for i in range(0, len(REALS), 2)] + ["services_dump_real_values"]
    for address in [0, 1, 16, 0x1234, 0x7FFF12345678, 2**64 - 1]:
        requests += [f"services_dump_pointer {address} {short}" for short in (0, 1)]
    return requests


def format_requests() -> list[str]:
    """Text and FormatJson with several indents."""
    return [f"services_dump_format {indent} {hex_of(text)}" for text in FORMATS for indent in (0, 1, 3, -2)]


def split_requests(rng: random.Random) -> list[str]:
    """SplitJson over fixed dumps and random token soups."""
    tokens = [b'"k"', b'"v"', b': ', b', ', b'{', b'}', b'[', b']', b'1', b'"', b'x', b' ', b'""']
    soups = [b"".join(rng.choice(tokens) for _ in range(rng.randint(1, 12))) for _ in range(150)]
    return [f"services_dump_split {hex_of(text)}" for text in DUMPS + soups]


def reader_requests(rng: random.Random) -> list[str]:
    """ProcessStreamName / ProcessFieldName / InitValue / InitRealValues at many positions."""
    requests = []
    for text in DUMPS[:8] + [b', "Gap": 1', b'"Value": 7}', b", ", b"[1, 2, 3]"]:
        positions = sorted({INT_MIN, -1, 0, 1, 2, 3, len(text), len(text) + 1, INT_MAX})
        for name in [b"className", b"Location", b"k", b"myValue", b"Gap", b"", b"Value\x00x"]:
            requests += [f"services_dump_process {kind} {hex_of(text)} {hex_of(name)} {pos}"
                         for kind in ("stream", "field") for pos in positions]
        requests += [f"services_dump_init_value {hex_of(text)} {pos}" for pos in positions]
    real_texts = [b"[1, 2, 3]", b"[1, 2, 3", b"[, 2]", b"[1e300, -0, 5e-324]", b"x[1]", b"[1]", b"[1, bad, 3]",
                  b"[1.5, 2.5] tail]", b"[", b"[]", b'"Location": [1, 2, 3]', b"[inf, nan, 1]"]
    for text in real_texts:
        for pos in sorted({INT_MIN, -1, 0, 1, 2, len(text), len(text) + 1}):
            requests += [f"services_dump_init_reals {hex_of(text)} {pos} {count}" for count in range(5)]
    for _ in range(60):
        text = rng.choice(DUMPS)
        requests.append(f"services_dump_init_value {hex_of(text)} {rng.randint(-2, len(text) + 2)}")
    return requests
