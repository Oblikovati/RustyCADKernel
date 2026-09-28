"""Requests for Standard_ReadBuffer and Standard_ReadLineBuffer (operation prefix `services_`).

Streams are byte strings that may fail after a given number of bytes (badbit on the
native side, an io::Error on the Rust side).

Excluded inputs (native undefined behavior, never probed):
- a zero chunk length (Init divides by it);
- ReadChunk<uint32_t> with a chunk shorter than 4 bytes (reads past the chunk);
- reading chunks after Init raised Standard_ProgramError.
"""
import random

LINE_TEXTS = [
    b"", b"a", b"a\n", b"a\nb", b"a\r\nb\r\n", b"\n\n\n", b"\r\n", b"\r", b"a\rb\n", b"\n",
    b"first line\nsecond line\nthird", b"x" * 50 + b"\n" + b"y" * 9, b"nul\x00inside\n", b"\xc3\xa9t\xc3\xa9\n",
    b"abc\\\ndef\n", b"abc\\\r\ndef\r\n", b"a\\b\n", b"1/2/3\\\n4/5/6\n", b"end\\", b"end\\\r", b"\\\n\\\n",
    b"ab\\\ncd\\\nef\ngh\n", b"a\\\r\nb\\\nc\r\n", b"trailing\r", b"\\\\\n",
]
LINE_SIZES = [0, 1, 2, 3, 4, 5, 7, 8, 16, 4096]
CLEARED_TEXT = b"ab\\\ncd\nef\\\ngh\n"
FAILING_TEXT = b"line one\nline two\n"


def readers_requests(rng: random.Random) -> list[str]:
    """ReadBuffer and ReadLineBuffer requests; readers_requests(random.Random(1))."""
    return read_buffer_requests(rng) + read_line_fixed() + [read_line_random(rng) for _ in range(250)]


def hex_of(data: bytes) -> str:
    """Hex field as the probes decode it, "-" for no bytes; hex_of(b"A") == "41"."""
    return data.hex() if data else "-"


def read_buffer_requests(rng: random.Random) -> list[str]:
    """Chunked reads: truncation, partial payloads, short and failing streams, refills."""
    cases = [
        (12, 4, 0, "-", bytes(range(12)), "d"), (13, 4, 0, "-", bytes(range(13)), "d"),
        (13, 4, 1, "-", bytes(range(13)), "d"), (10, 3, 1, "-", bytes(range(10)), "d"),
        (12, 4, 0, "-", bytes(range(8)), "d"), (12, 4, 0, "6", bytes(range(12)), "d"),
        (0, 4, 0, "-", b"abcd", "d"), (-5, 4, 0, "-", b"abcd", "d"), (-5, 4, 1, "-", b"abcd", "d"),
        (3, 4, 0, "-", b"abc", "d"), (3, 4, 1, "-", b"abc", "d"), (16, 8, 0, "-", bytes(range(16)), "u"),
        (12, 4, 0, "-", bytes(range(12)), "u"), (4096, 4096, 0, "-", bytes(4096), "d"),
        (8, 4097, 0, "-", bytes(8), "d"), (8, 5000, 1, "-", bytes(8), "d"),
        (10000, 1000, 0, "-", bytes(i % 251 for i in range(10000)), "d"),
        (10000, 1000, 1, "-", bytes(i % 251 for i in range(10000)), "d"),
        (9000, 1000, 0, "5000", bytes(i % 7 for i in range(9000)), "d"),
        (5000, 7, 1, "-", bytes(i % 13 for i in range(5000)), "d"),
    ]
    for _ in range(40):
        chunk = rng.choice([1, 2, 3, 4, 5, 8, 12, 100, 1024])
        data = bytes(rng.randrange(256) for _ in range(rng.randint(0, 300)))
        length = rng.choice([len(data), len(data) + rng.randint(1, 9), max(len(data) - 5, 0), -len(data)])
        fail = rng.choice(["-", "-", str(rng.randint(0, max(len(data), 1)))])
        mode = "u" if chunk >= 4 and rng.random() < 0.3 else "d"
        cases.append((length, chunk, rng.randint(0, 1), fail, data, mode))
    return [f"services_read_buffer {n} {c} {p} {f} {hex_of(d)} {m}" for n, c, p, f, d, m in cases]


def read_line_fixed() -> list[str]:
    """Every fixed text through every buffer size, with and without multiline modes."""
    requests = []
    for text in LINE_TEXTS:
        for size in LINE_SIZES:
            for multiline, gap in [(0, 1), (1, 1), (1, 0)]:
                requests.append(f"services_read_line {size} {multiline} {gap} - - {hex_of(text)}")
    requests += [f"services_read_line 4 1 1 - {clear} {hex_of(CLEARED_TEXT)}" for clear in "12"]
    requests += [f"services_read_line 3 0 1 {fail} - {hex_of(FAILING_TEXT)}" for fail in "0379"]
    return requests


def read_line_random(rng: random.Random) -> str:
    """A random text over the bytes the scanner treats specially."""
    alphabet = [b"a", b"b", b"\n", b"\r", b"\\", b" ", b"\x00", b"\xc3", b"\xa9"]
    text = b"".join(rng.choice(alphabet) for _ in range(rng.randint(0, 60)))
    fail = rng.choice(["-", "-", "-", str(rng.randint(0, 60))])
    clear = rng.choice(["-", "-", "-", str(rng.randint(1, 4))])
    size = rng.choice([1, 2, 3, 4, 5, 6, 8, 11, 64])
    return f"services_read_line {size} {rng.randint(0, 1)} {rng.randint(0, 1)} {fail} {clear} {hex_of(text)}"
