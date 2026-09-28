"""Deterministic requests for TCollection_ExtendedString and TCollection_HExtendedString.

Formats follow tools/oracle/tkernel_extended_string_api_probe.cxx: code-unit strings are hex
of little-endian UTF-16 units ("-" when empty), "null" is a null char16_t pointer, byte
strings are hex, reals are decimal IEEE-754 bit patterns.

Inputs that are undefined behavior in OCCT are never sent:
- UTF-8 whose announced sequence runs into the terminating NUL before a decoded zero ends
  the scan: NCollection_UtfIterator then reads past the string (the port treats such input
  as not UTF-8 and widens the bytes);
- (pointer, length) arguments naming more units than the buffer holds, a null pointer with
  a positive length for Cat, and a negative Cat length whose total stays >= 0 (OCCT copies
  the whole string into the shorter result buffer; the port raises Standard_OutOfRange);
- Remove with theWhere + theHowMany overflowing int (the native check wraps and the removal
  corrupts memory; the port raises Standard_OutOfRange), and lengths or widths large enough
  to exhaust memory;
- aliased AssignCat/Prepend pointers outside the string, a null Token separator pointer and
  a null handle argument (Rust slices and references are never null).
"""
import random
import struct

INT_MIN, INT_MAX = -2147483648, 2147483647

STRINGS = [
    "", "a", "Hello World", "  pad  ", "Hello\0World", [0x41, 0xD800, 0x42], [0xDC00, 0x41],
    [0xD83D, 0xDE00], [0xE9, 0x20AC, 0x4E2D], [0xFFFF, 0xFFFE], "abcabcab", "aAbBzZ@[`{",
    [0xFF, 0x100, 0x7F], "123", "  -42abc", "3.5e2x", "1.5", " 7", "0x1F", "nan", "inf", "-",
    "1e999", "99999999999", [0x663], "12\0 34", " \t tab sep\t", "; :,a;;b ,c", [0, 0x41],
]
UTF8 = [
    b"", b"A", b"Hello", "é".encode(), "€".encode(), "中文".encode(), "😀".encode(), "a😀b€".encode(),
    b"\xc0\x80", b"A\xc0\x80B", b"\xc1\xbf", b"\xe0\x80\x80Z", b"\xf0\x80\x80\x80", b"\x80", b"\xbf\xa9",
    b"\xc3A", b"\xe2\x82A", b"\xe2A\x82", b"\xed\xa0\x80", b"\xed\xbf\xbfZ", b"\xed\xa0\xbd\xed\xb8\x80",
    b"\xf4\x8f\xbf\xbf", b"\xf4\x90\x80\x80", b"\xf7\xbf\xbf\xbf", b"\xf8\x88\x80\x80\x80",
    b"\xfc\x84\x80\x80\x80\x80", b"\xfe\x80\x80\x80\x80\x80", b"\xff\xbf\xbf\xbf\xbf\xbf",
    b"\xf8AAAA", b"\xfcAAAAA", b"A\0B", b"\xe9t\xe9", b"ok\xf4\x90\x80\x80", b"\xc0\x80\xc3",
]
WIDE = [[], [0x41], [0x41, 0xE9, 0x20AC], [0x1F600, 0x10FFFF], [0x110000, 0x42], [0xD800, 0xDFFF, 0x43],
        [0xFFFFFFFF, 0x80000000, 0x44], [0xFFFE, 0xFFFF], [0x45, 0, 0x46]]
REALS = [0.0, -0.0, 1.0, -2.5, 3.14159265, 1e-5, 123456789.0, 1e300, 5e-324, float("inf"), float("-inf"),
         float("nan"), 0.1, 100000.0, 1e6]


def units_of(text) -> list[int]:
    """Code units of a str (UTF-16) or a unit list; units_of('a') == [97]."""
    if isinstance(text, str):
        encoded = text.encode("utf-16-le")
        return [int.from_bytes(encoded[i:i + 2], "little") for i in range(0, len(encoded), 2)]
    return list(text)


def u16(text) -> str:
    """Hex of little-endian UTF-16 units, '-' when empty; u16('a') == '6100'."""
    return "".join(f"{unit & 0xFF:02x}{unit >> 8:02x}" for unit in units_of(text)) or "-"


def raw(data: bytes) -> str:
    """Hex bytes, '-' when empty; raw(b'A') == '41'."""
    return data.hex() or "-"


def bits(value: float) -> int:
    """Decimal IEEE-754 bit pattern; bits(1.0) == 4607182418800017408."""
    return struct.unpack("<Q", struct.pack("<d", value))[0]


def overruns(data: bytes) -> bool:
    """True when OCCT's UTF-8 scan would read past the NUL of this C string."""
    text = data.split(b"\0")[0]
    offsets = [0, 0x3080, 0xE2080, 0x03C82080, 0xFA082080, 0x82082080]
    at = 0
    while at < len(text):
        lead = text[at]
        count = 0 if lead < 0xC0 else 1 if lead < 0xE0 else 2 if lead < 0xF0 else 3 if lead < 0xF8 else 4 if lead < 0xFC else 5
        if at + count >= len(text):
            return True
        value = 0
        for byte in text[at:at + count + 1]:
            value = ((value << 6) + byte) & 0xFFFFFFFF
        if (value - offsets[count]) & 0xFFFFFFFF == 0:
            return False
        at += count + 1
    return False


def indices(length: int) -> list[int]:
    """Boundary one-based indices around a string of `length` units."""
    return sorted({INT_MIN, -1, 0, 1, 2, length, length + 1, length + 2, INT_MAX})


def append_extended_string_api_requests(requests: list[str]) -> None:
    """Append this area's requests with its own seed; append_extended_string_api_requests(requests)."""
    requests.extend(area_requests(random.Random(20260929)))


def area_requests(rng: random.Random) -> list[str]:
    """Every request of this area, drawn from `rng`; area_requests(random.Random(1))."""
    # Imported here: the edit corpus reuses this module's helpers, so a top-level import
    # would be circular.
    from tkernel_extended_string_api_corpus_edits import edit_requests, handle_requests
    return construction_requests(rng) + query_requests(rng) + edit_requests(rng) + handle_requests()


def random_utf8(rng: random.Random) -> bytes:
    """A random byte string biased towards UTF-8 lead and continuation bytes."""
    pool = [0x41, 0x7A, 0x00, 0x80, 0xBF, 0xC2, 0xC3, 0xDF, 0xE0, 0xE2, 0xED, 0xEF, 0xF0, 0xF4, 0xF5, 0xF8, 0xFC, 0xFF]
    return bytes(rng.choice(pool) if rng.random() < 0.7 else rng.randrange(256) for _ in range(rng.randrange(1, 9)))


def construction_requests(rng: random.Random) -> list[str]:
    """Every constructor with its boundary inputs."""
    corpus = UTF8 + [data for data in (random_utf8(rng) for _ in range(400)) if not overruns(data)]
    out = ["extended_string_new empty"]
    for data in corpus:
        out += [f"extended_string_new cstr {raw(data)} {flag}" for flag in (0, 1)]
        out += [f"extended_string_new ascii {raw(data)} {flag}" for flag in (0, 1)]
    for text in STRINGS:
        out += [f"extended_string_new u16 {u16(text)}", f"extended_string_new view {u16(text)}"]
        units = units_of(text)
        out += [f"extended_string_new span {u16(text)} {n}" for n in sorted({0, len(units), len(units) // 2, -1, INT_MIN})]
    out += [f"extended_string_new span null {n}" for n in (0, 3, -1)]
    out += ["extended_string_new wide " + ("".join(v.to_bytes(4, "little").hex() for v in w) or "-") for w in WIDE]
    out += [f"extended_string_new char {c}" for c in (0, 65, 127, 128, 233, 255)]
    out += [f"extended_string_new unit {c}" for c in (0, 65, 255, 256, 0xD800, 0xFFFF)]
    out += [f"extended_string_new fill {n} {c}" for n in (INT_MIN, -1, 0, 1, 3) for c in (0, 45, 0x20AC)]
    out += [f"extended_string_new int {v}" for v in (0, 1, -1, 42, INT_MIN, INT_MAX)]
    out += [f"extended_string_new real {bits(v)}" for v in REALS]
    return out


def query_requests(rng: random.Random) -> list[str]:
    """Conversions, numeric parsing, comparisons, searches, tokens and element access."""
    units = [[rng.choice([0, 0x41, 0x7F, 0xE9, 0x20AC, 0xD83D, 0xDE00, 0xD800, 0xDC00, 0xFFFF]) for _ in range(rng.randrange(0, 7))]
             for _ in range(200)]
    out = [f"extended_string_convert {u16(text)}" for text in STRINGS + units]
    out += [f"extended_string_numeric {u16(text)}" for text in STRINGS]
    pairs = [(a, b) for a in STRINGS[:16] for b in ["", "a", "Hello", "World", "ab", "abc", "b", "Hello World!", [0xFFFF], "hello world"]]
    out += [f"extended_string_whole {u16(a)} {u16(b)}" for a, b in pairs]
    for a, b in pairs[::3]:
        count = len(units_of(b))
        out += [f"extended_string_span {u16(a)} {u16(b)} {n}" for n in sorted({0, count, max(count - 1, 0), -1, INT_MIN})]
        out += [f"extended_string_span {u16(a)} null {n}" for n in (0, 1, 5, -1)]
    separators = [" ", " \t", "; :,", "", "\0 ", "a"]
    out += [f"extended_string_token {u16(t)} {u16(s)} {w}" for t in STRINGS for s in separators
            for w in (INT_MIN, -1, 0, 1, 2, 3, 5, INT_MAX)]
    for text in STRINGS[:14]:
        n = len(units_of(text))
        out += [f"extended_string_locate {u16(text)} {u16(s)} {f} {t}" for s in ("", "lo", "a ", "xyz")
                for f in (0, 1, 2, n) for t in (0, 1, n, n + 1)]
        out += [f"extended_string_value {u16(text)} {i}" for i in indices(n)]
    return out
