"""Deterministic requests for TCollection_AsciiString, TCollection_HAsciiString and TCollection.

Strings are hex fields (`-` when empty) so NUL and high-bit bytes survive; integers are
decimal and reals are hex bit patterns. The corpus covers empty strings, embedded and
leading/trailing NULs, high-bit and malformed UTF-8 bytes, positions 0/1/N/N+1/N+2 and
INT_MIN/INT_MAX, zero and negative counts, and every numeric parsing edge.

Inputs excluded because OCCT's behavior is undefined for them (never probed):
- `TCollection_AsciiString(int, char)` / `HAsciiString(int, char)` with a negative length
  (writes before the buffer);
- `Remove(where, n)` with `where < 1` or `n < 0` when `where + n <= Length() + 1`, and any
  `where + n` outside the int range (out-of-bounds writes, signed overflow);
- `SetValue(where, str)` with `where > Length() + 1` (leaves uninitialized bytes);
- `Copy(own pointer + k, n)` with `0 < k < n` (overlapping memcpy), and self pointers that
  do not point inside the string for `AssignCat`/`Insert` (freed-buffer reads);
- `UsefullLength` on a string whose last UTF-8 lead byte announces continuation bytes
  past the terminator (reads allocation padding);
- `Read`/`operator>>` on input holding only whitespace (the stack buffer stays uninitialized);
- null handles where HAsciiString dereferences without a check (all but `IsDifferent` and
  the two `IsSameString`), and `SetValue(where, h)` with `h` the object itself (reads a
  reallocated buffer).
"""
import random
import struct

INT_MIN, INT_MAX = -(2**31), 2**31 - 1
TEXTS = [
    b"", b"a", b"A", b"abc", b"Hello World", b"hello world", b"  padded\t\n", b"\t\v\f\r x \n",
    b"a\0b", b"\0", b"\0abc", b"abc\0", b"ab\0\0cd", b"\xff\xfe", b"caf\xc3\xa9", b"\xe2\x82\xac!",
    b"\xf0\x9f\x98\x80 ", b"\xc0\x80tail", b"\x80abc", b"MiXeD CaSe 123", b"12345", b"-42", b"+7",
    b"  -2147483649", b"2147483648", b"-9223372036854775809", b"99999999999999999999999", b"3.14",
    b"1e10", b".5", b"-0", b"inf", b"-nan", b"0x1A", b"1.5e", b"12.", b"12e3", b"12\0.5", b"1.5\0x",
    b"aaab", b"abababab", b"a b\tc,d;e", b",,a,,b,,", b"This is a test string", b"~ !\x7f",
]
SPANS = [b"", b"a", b"b", b"ab", b"aab", b"abc", b"\0", b"a\0", b"\0b", b"World", b" ", b"\xff", b"xyz!", b"is"]
BYTES = [0, 1, 9, 32, 48, 65, 97, 122, 127, 128, 195, 255]
REALS = [0.0, -0.0, 1.0, -1.0, 3.3, 0.1, 1e-5, 1e-4, 123456.0, 999999.5, 1e6, 1e21, -1.5e-300,
         5e-324, 2.2250738585072014e-308, 1.7976931348623157e308, float("inf"), float("-inf"),
         float("nan"), -float("nan"), 0.000123456789, 1234567.0]
WIDE = [[], [65], [0x20AC, 0x1F600], [65, 0, 66], [0xD800, 66], [0xDFFF], [0x110000, 67], [0x7FFFFFFF],
        [0x80000000, 68], [0xFFFFFFFF], [0x7F, 0x80, 0x7FF, 0x800, 0xFFFF, 0x10000, 0x10FFFF]]
UTF16 = [[], [0x41], [0xE9, 0x20AC], [0xD83D, 0xDE00], [0xD800], [0xDC00, 0x41], [0x41, 0, 0x42], [0xFF, 0x100]]
REPLACEMENTS = [0, 63, 0xE9]


def append_ascii_string_api_requests(requests: list[str]) -> None:
    """Append this area's requests with its own seed; append_ascii_string_api_requests(requests)."""
    requests.extend(area_requests(random.Random(20260928)))


def area_requests(rng: random.Random) -> list[str]:
    """Every request of this area, drawn from `rng`; area_requests(random.Random(1))."""
    texts = TEXTS + [random_text(rng) for _ in range(24)]
    requests: list[str] = []
    for build in (make_requests, concatenation_requests, position_requests, removal_requests, layout_requests,
                  query_requests, value_requests, stream_requests, handle_requests, prime_requests):
        requests.extend(build(texts, rng))
    return requests


def random_text(rng: random.Random) -> bytes:
    """A short string over a biased alphabet that includes NUL, spaces and high-bit bytes."""
    alphabet = b"aAbB  ,;\t\x00\x80\xc3\xa9\xff019.-e"
    return bytes(rng.choice(alphabet) for _ in range(rng.randint(0, 12)))


def h(data: bytes) -> str:
    """Hex field; '-' for empty."""
    return data.hex() or "-"


def bits(value: float) -> str:
    """IEEE-754 bit pattern field."""
    return format(struct.unpack("<Q", struct.pack("<d", value))[0], "x")


def wide(units: list[int]) -> str:
    """`<count> <u32>...` field."""
    return " ".join([str(len(units))] + [str(unit) for unit in units])


def utf16(units: list[int]) -> str:
    """4-digit UTF-16 units field."""
    return "".join(format(unit, "04x") for unit in units) or "-"


def positions(length: int) -> list[int]:
    """One-based boundary positions around a string of `length` bytes."""
    return sorted({INT_MIN, -1, 0, 1, 2, length // 2, length, length + 1, length + 2, INT_MAX})


def make_requests(texts: list[bytes], rng: random.Random) -> list[str]:
    out = ["ascii_make default", "ascii_make empty_string"]
    for text in texts:
        out += [f"ascii_make cstring {h(text)}", f"ascii_make view {h(text)}", f"ascii_make copy {h(text)}"]
        out += [f"ascii_make ptr_len {h(text)} {n}" for n in (INT_MIN, -1, 0, 1, 3, len(text), len(text) + 5)]
        out += [f"ascii_make join_char {h(text)} {b}" for b in (0, 33, 255)]
        out += [f"ascii_make join_cstring {h(text)} {h(span)}" for span in SPANS[:6]]
        out.append(f"ascii_make join {h(text)} {h(rng.choice(texts))}")
    out += [f"ascii_make char {b}" for b in BYTES]
    out += [f"ascii_make filled {n} {b}" for n in (0, 1, 7) for b in (0, 42, 200)]
    out += [f"ascii_make int {n}" for n in (INT_MIN, -1, 0, 7, -12, 123456, INT_MAX)]
    out += [f"ascii_make real {bits(value)}" for value in REALS]
    out += [f"ascii_make real {bits(rng.uniform(-1e7, 1e7) * 10.0 ** rng.randint(-12, 12))}" for _ in range(40)]
    out += [f"ascii_make wide {wide(units)}" for units in WIDE]
    out += [f"ascii_make extended {utf16(units)} {r}" for units in UTF16 for r in REPLACEMENTS]
    return out


def concatenation_requests(texts: list[bytes], rng: random.Random) -> list[str]:
    out = []
    for text in texts:
        s = h(text)
        for member in ("assign_cat_char", "plus_assign_char"):
            out += [f"ascii_edit {member} {s} {b}" for b in (0, 65, 255)]
        for member in ("cat_char", "plus_char"):
            out += [f"ascii_make {member} {s} {b}" for b in (0, 65, 255)]
        for kind in ("int", "real"):
            value = rng.choice([INT_MIN, -3, 0, 42, INT_MAX]) if kind == "int" else bits(rng.choice(REALS))
            out += [f"ascii_edit assign_cat_{kind} {s} {value}", f"ascii_edit plus_assign_{kind} {s} {value}",
                    f"ascii_make cat_{kind} {s} {value}", f"ascii_make plus_{kind} {s} {value}"]
        for span in SPANS[:8]:
            out += [f"ascii_edit assign_cat_{kind} {s} {h(span)}" for kind in ("span", "cstring", "view", "string")]
            out += [f"ascii_edit plus_assign_{kind} {s} {h(span)}" for kind in ("cstring", "string")]
            out += [f"ascii_make cat_{kind} {s} {h(span)}" for kind in ("span", "cstring", "view", "string")]
            out += [f"ascii_make plus_{kind} {s} {h(span)}" for kind in ("cstring", "string")]
        units = rng.choice(UTF16)
        out += [f"ascii_edit assign_cat_extended {s} {utf16(units)} {r}" for r in REPLACEMENTS]
        out += [f"ascii_make cat_extended {s} {utf16(units)} {r}" for r in REPLACEMENTS]
        out += [f"ascii_edit plus_assign_extended {s} {utf16(units)}", f"ascii_make plus_extended {s} {utf16(units)}"]
        units = rng.choice(WIDE)
        out += [f"ascii_{group} {member} {s} {wide(units)}" for group, member in
                (("edit", "assign_cat_wide"), ("edit", "plus_assign_wide"), ("make", "cat_wide"), ("make", "plus_wide"))]
        out += [f"ascii_edit assign_cat_self {s} {k} {n}" for k, n in own_spans(len(text), False)]
        out.append(f"ascii_edit prepend {s} {h(rng.choice(SPANS))}")
    return out


def own_spans(length: int, allow_empty: bool) -> list[tuple[int, int]]:
    """(offset, count) pointers inside a string: offset < length, offset + count <= length."""
    spans = [(k, n) for k in range(length) for n in range(0 if allow_empty else 1, length - k + 1)]
    return spans[:: max(1, len(spans) // 6)]


def position_requests(texts: list[bytes], rng: random.Random) -> list[str]:
    out = []
    for text in texts:
        s, length = h(text), len(text)
        span = rng.choice(SPANS)
        for p in positions(length):
            out += [f"ascii_edit insert_char {s} {p} {rng.choice(BYTES)}", f"ascii_edit set_value_char {s} {p} 88",
                    f"ascii_edit set_value_char {s} {p} 0", f"ascii_edit insert_after {s} {p} {h(span)}",
                    f"ascii_edit insert_before {s} {p} {h(span)}"]
            out += [f"ascii_edit insert_{kind} {s} {p} {h(span)}" for kind in ("span", "cstring", "string")]
            if p <= length + 1:
                out += [f"ascii_edit set_value_{kind} {s} {p} {h(span)}" for kind in ("span", "string")]
            out += [f"ascii_edit insert_self {s} {p} {k} {n}" for k, n in own_spans(length, True)[:2]]
    return out


def removal_requests(texts: list[bytes], rng: random.Random) -> list[str]:
    out = []
    for text in texts:
        s, length = h(text), len(text)
        for p in positions(length):
            for n in (INT_MIN, -1, 0, 1, 2, length, INT_MAX):
                total = p + n
                defined_throw = total > length + 1 and INT_MIN <= total <= INT_MAX
                if (p >= 1 and n >= 0 and total <= INT_MAX) or defined_throw:
                    out.append(f"ascii_edit remove {s} {p} {n}")
            out += [f"ascii_edit trunc {s} {p}", f"ascii_edit split {s} {p}"]
        out += [f"ascii_edit remove_all {s} {b}" for b in (0, 97, 65, 32)]
        out += [f"ascii_edit remove_all_case {s} {b} {f}" for b in (97, 65, 195) for f in (0, 1)]
        span = rng.choice(texts)
        out += [f"ascii_edit copy_{kind} {s} {h(span)}" for kind in ("span", "cstring", "view", "string")]
        out += [f"ascii_edit copy_self {s} 0 {n}" for n in range(length + 1)]
        out += [f"ascii_edit copy_self {s} {k} {n}" for k in range(1, length) for n in range(0, min(k, length - k) + 1)]
        out += [f"ascii_edit {member} {s} {h(span)}" for member in ("move", "swap")]
        out.append(f"ascii_edit clear {s}")
    return out


def layout_requests(texts: list[bytes], rng: random.Random) -> list[str]:
    out = []
    for text in texts:
        s = h(text)
        for member in ("center", "left_justify", "right_justify"):
            out += [f"ascii_edit {member} {s} {w} {rng.choice([42, 0, 200])}"
                    for w in (INT_MIN, -1, 0, len(text), len(text) + 1, len(text) + 4, len(text) + 7)]
        out += [f"ascii_edit {member} {s}" for member in ("left_adjust", "right_adjust", "upper", "lower", "capitalize")]
        out += [f"ascii_edit change_all {s} {a} {b} {f}" for a, b in ((97, 111), (65, 0), (0, 33), (195, 97))
                for f in (0, 1)]
    return out


def query_requests(texts: list[bytes], rng: random.Random) -> list[str]:
    out = []
    for text in texts:
        s, length = h(text), len(text)
        for span in SPANS:
            out += [f"ascii_query {member} {s} {h(span)}" for member in
                    ("search_span", "search_cstring", "search_string", "search_from_end_span",
                     "search_from_end_cstring", "compare")]
        for p in positions(length):
            out.append(f"ascii_query value {s} {p}")
            for q in (p, length, p + 1, 0):
                what = h(rng.choice(SPANS))
                out += [f"ascii_query sub_string {s} {p} {q}", f"ascii_query location {s} {what} {p} {q}",
                        f"ascii_query location_char {s} {rng.choice([-1, 0, 1, 2])} {rng.choice([97, 0, 32])} {p} {q}",
                        f"ascii_query in_set {s} {what} {p} {q}", f"ascii_query not_in_set {s} {what} {p} {q}"]
        for separators in (b" \t", b",;", b"", b"a", b",\0b", b"\xff"):
            out += [f"ascii_query token {s} {h(separators)} {n}" for n in (INT_MIN, 0, 1, 2, 3, 9)]
        out.append(f"ascii_query token_default {s}")
        out.append(f"ascii_query compare {s} {h(rng.choice(texts))}")
    return out


def utf8_scan_is_defined(text: bytes) -> bool:
    """False when `UsefullLength`'s permissive UTF-8 reader would read past the terminator."""
    buffer, position = text + b"\0", 0
    while True:
        lead = buffer[position]
        trailing = 0 if lead < 0xC0 else 1 if lead < 0xE0 else 2 if lead < 0xF0 else 3 if lead < 0xF8 \
            else 4 if lead < 0xFC else 5
        if position + trailing >= len(buffer):
            return False
        code = 0
        for byte in buffer[position:position + trailing + 1]:
            code = ((code << 6) + byte) & 0xFFFFFFFF
        offsets = [0, 0x3080, 0xE2080, 0x3C82080, 0xFA082080, 0x82082080]
        if (code - offsets[trailing]) & 0xFFFFFFFF == 0:
            return True
        position += trailing + 1
        if position >= len(buffer):
            return False


def value_requests(texts: list[bytes], rng: random.Random) -> list[str]:
    out = []
    for text in texts + [bytes([rng.choice(b"\x00\x80\xc3\xe2\xf0\xfc a")]) + random_text(rng) for _ in range(60)]:
        s = h(text)
        if utf8_scan_is_defined(text):
            out.append(f"ascii_query properties {s}")
        out += [f"ascii_query {member} {s}" for member in ("integer_value", "real_value", "print")]
    return out


def stream_requests(texts: list[bytes], rng: random.Random) -> list[str]:
    inputs = [b"word", b"  lead", b"two words", b"\t\ntab\x0bvt rest", b"a\0b c", b"\xff\xfe x", b"end\n",
              b"x" * 8189, b"y" * 8190 + b" z", b"q" * 9000]
    inputs += [text for text in texts if text.strip(b" \t\n\v\f\r")]
    return [f"ascii_stream {member} {h(data)}" for data in inputs for member in ("read", "extract")]


def handle_requests(texts: list[bytes], rng: random.Random) -> list[str]:
    out = ["ascii_h_make default"]
    for text in texts:
        s = h(text)
        out += [f"ascii_h_make {member} {s}" for member in ("cstring", "string", "moved_string", "handle")]
        out += handle_edit_requests(s, len(text), rng) + handle_query_requests(text, rng)
    out += [f"ascii_h_make char {b}" for b in BYTES] + [f"ascii_h_make filled {n} 35" for n in (0, 3)]
    out += [f"ascii_h_make int {n}" for n in (INT_MIN, 0, 99)] + [f"ascii_h_make real {bits(v)}" for v in REALS[:8]]
    out += [f"ascii_h_make extended {utf16(units)} {r}" for units in UTF16 for r in REPLACEMENTS]
    return out


def handle_edit_requests(s: str, length: int, rng: random.Random) -> list[str]:
    span = h(rng.choice(SPANS))
    out = [f"ascii_h_edit {member} {s}" for member in
           ("assign_cat_self", "capitalize", "upper", "lower", "clear", "left_adjust", "right_adjust",
            "prepend_self", "token_default")]
    out += [f"ascii_h_edit {member} {s} {span}" for member in
            ("assign_cat", "assign_cat_handle", "cat", "cat_handle", "prepend")]
    for p in positions(length):
        out += [f"ascii_h_edit {member} {s} {p} {span}" for member in
                ("insert", "insert_handle", "insert_after", "insert_before")]
        out += [f"ascii_h_edit insert_char {s} {p} 64", f"ascii_h_edit insert_self {s} {p}",
                f"ascii_h_edit set_value_char {s} {p} 0", f"ascii_h_edit set_value_char {s} {p} 90",
                f"ascii_h_edit split {s} {p}", f"ascii_h_edit trunc {s} {p}", f"ascii_h_edit sub_string {s} {p} {length}"]
        if p <= length + 1:
            out += [f"ascii_h_edit set_value {s} {p} {span}", f"ascii_h_edit set_value_handle {s} {p} {span}"]
        if 1 <= p <= length or length + 1 < p + 1 <= INT_MAX:
            out.append(f"ascii_h_edit remove {s} {p} 1")
    for member in ("center", "left_justify", "right_justify"):
        out += [f"ascii_h_edit {member} {s} {w} 46" for w in (INT_MIN, -1, 0, length + 3)]
    out += [f"ascii_h_edit change_all {s} 97 65 0", f"ascii_h_edit remove_all {s} 32",
            f"ascii_h_edit remove_all_case {s} 65 0", f"ascii_h_edit token {s} {h(b' ,')} 2"]
    return out


def handle_query_requests(text: bytes, rng: random.Random) -> list[str]:
    s, length = h(text), len(text)
    other = h(rng.choice(TEXTS))
    out = [f"ascii_h_query {member} {s} {handle}" for handle in (s, other, "null")
           for member in ("is_different", "is_same_string")]
    out += [f"ascii_h_query is_same_string_case {s} {handle} {f}" for handle in (s, other, "null") for f in (0, 1)]
    out += [f"ascii_h_query {member} {s} {handle}" for handle in (s, other)
            for member in ("is_less", "is_greater", "is_same_state", "search_handle", "search_from_end_handle")]
    out += [f"ascii_h_query {member} {s} {h(span)}" for span in SPANS[:5] for member in ("search", "search_from_end")]
    for p in positions(length):
        out += [f"ascii_h_query {member} {s} {h(rng.choice(SPANS))} {p} {length}"
                for member in ("in_set", "not_in_set", "location")]
        out += [f"ascii_h_query location_char {s} 1 97 {p} {length}", f"ascii_h_query value {s} {p}"]
    out += [f"ascii_h_query {member} {s}" for member in ("integer_value", "real_value")]
    if utf8_scan_is_defined(text):
        out.append(f"ascii_h_query properties {s}")
    return out


# Equal-length pairs that differ only after a shared NUL: `strncmp` members call them the same.
NUL_PAIRS = [(b"a\0b", b"a\0c"), (b"\0x", b"\0y"), (b"ab\0\0cd", b"ab\0\0xy"), (b"a\0b", b"a\0b"),
             (b"abc", b"abd"), (b"a\0b", b"ab\0"), (b"ABC", b"abc")]


def nul_pair_requests() -> list[str]:
    """Every comparison of the NUL pairs, through AsciiString and HAsciiString."""
    out = [f"ascii_query compare {h(first)} {h(second)}" for first, second in NUL_PAIRS]
    for first, second in NUL_PAIRS:
        out += [f"ascii_h_query {member} {h(first)} {h(second)}" for member in
                ("is_different", "is_same_string", "is_same_state", "is_less", "is_greater")]
        out += [f"ascii_h_query is_same_string_case {h(first)} {h(second)} {f}" for f in (0, 1)]
    return out


def prime_requests(texts: list[bytes], rng: random.Random) -> list[str]:
    values = [INT_MIN, INT_MIN + 1, -3, -2, -1, 0, 1, 100, 101, 102, 1009, 1010, 65003, 995328, 995329,
              1019215873, 2038431744, 2038431745, 2038431746, INT_MAX]
    values += [rng.randint(INT_MIN, INT_MAX) for _ in range(20)]
    return [f"ascii_next_prime {value}" for value in values] + nul_pair_requests()
