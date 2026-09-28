"""Deterministic requests for Standard scalar, character, hashing, time, GUID and atomic helpers.

Doubles and floats travel as IEEE-754 bit patterns (16/8 hex digits) and come back the same
way, so signed zeros, subnormals, infinities and NaN payloads are compared exactly.

Inputs excluded because the C++ result is undefined behavior (the Rust choice is documented
in docs/tkernel-progress.md and tested in crates/tkernel/tests/standard_values_api_behavior):
- RealToInt(NaN): NaN reaches `static_cast<int>`.
- Abs(INT_MIN), Square(int) beyond +-46340: signed overflow.
- Modulus(x, 0) and Modulus(INT_MIN, -1): division traps.
- the hash templates with a length beyond the object, and negative lengths: out-of-bounds reads.
- Standard_GUID(const char16_t*) with fewer than 36 units: out-of-bounds read.
Atof requests are ASCII only: the Rust `strtod` takes `&str`.
"""
import random
import re
import struct

PREFIX = "standard_values_"
REAL_UNARY = ["acos", "acos_approx", "asin", "atan", "atanh", "acosh", "asinh", "cosh", "sinh", "tanh",
              "cos", "sin", "tan", "exp", "log", "log10", "sqrt", "abs", "ceiling", "floor", "integer_part",
              "round", "epsilon", "real_part", "square"]
REAL_BINARY = ["atan2", "next_after", "sign", "max", "min", "pow", "is_equal"]
INT_MIN, INT_MAX = -2**31, 2**31 - 1
GUID_FORMAT = re.compile(rb"[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}")
HASH_TYPES = ["char", "int", "uint16", "int64", "double"]


def bits(value: float) -> str:
    """IEEE-754 bits of a double as 16 hex digits; bits(1.0) == '3ff0000000000000'."""
    return f"{struct.unpack('<Q', struct.pack('<d', value))[0]:016x}"


def bits32(value: float) -> str:
    """IEEE-754 bits of a float as 8 hex digits; bits32(1.0) == '3f800000'."""
    return f"{struct.unpack('<I', struct.pack('<f', value))[0]:08x}"


def edge_doubles() -> list[str]:
    """Signed zeros, subnormals, limits, infinities, NaN payloads and clamp boundaries."""
    raw = ["0000000000000000", "8000000000000000", "0000000000000001", "8000000000000001",
           "000fffffffffffff", "0010000000000000", "8010000000000000", "7fefffffffffffff",
           "ffefffffffffffff", "7ff0000000000000", "fff0000000000000", "7ff8000000000000",
           "fff8000000000000", "7ff8000000000123", "7ff0000000000001"]
    values = [1.0, -1.0, 0.5, -0.5, 2.0, 2.5, -2.5, 3.141592653589793, 1e-300, 1e300, 709.78, 710.0,
              -745.2, 0.9999999999999999, 1.0000000000000002, -0.9999999999999999, 0.49999999999999994,
              4503599627370495.5, 2147483647.0, 2147483647.5, 2147483648.0, -2147483648.0, -2147483648.5,
              -2147483649.0, 3.4028234663852886e38, 3.4028235677973362e38, 3.4028235677973366e38,
              -3.4028235677973366e38, 1.401298464324817e-45, 7.006492321624085e-46, 1e-50, 0.1, -0.7]
    return raw + [bits(v) for v in values]


def random_doubles(rng: random.Random, count: int) -> list[str]:
    """Random finite doubles on several scales plus random bit patterns."""
    scaled = [bits(rng.uniform(-1.0, 1.0) * 10.0 ** rng.randint(-8, 8)) for _ in range(count)]
    patterns = [f"{rng.getrandbits(64):016x}" for _ in range(count // 4)]
    return scaled + patterns


def real_requests(rng: random.Random) -> list[str]:
    """Unary and binary Standard_Real functions over edge and random doubles."""
    doubles = edge_doubles() + random_doubles(rng, 40)
    requests = [f"{PREFIX}real1 {name} {value}" for name in REAL_UNARY for value in doubles]
    pairs = [(a, b) for a in edge_doubles() for b in edge_doubles()[::3]]
    pairs += [(rng.choice(doubles), rng.choice(doubles)) for _ in range(60)]
    requests += [f"{PREFIX}real2 {name} {a} {b}" for name in REAL_BINARY for a, b in pairs]
    requests += [f"{PREFIX}real_to_int {v}" for v in doubles if not is_nan(v)]
    requests += [f"{PREFIX}real_to_short_real {v}" for v in doubles]
    ints = [0, 1, -1, INT_MIN, INT_MAX, 2**24 + 1, -(2**24) - 1] + [rng.randint(INT_MIN, INT_MAX) for _ in range(20)]
    return requests + [f"{PREFIX}int_to_real {i}" for i in ints]


def is_nan(pattern: str) -> bool:
    """True for a NaN bit pattern; is_nan('7ff8000000000000')."""
    value = int(pattern, 16) & 0x7FFFFFFFFFFFFFFF
    return value > 0x7FF0000000000000


def short_real_requests(rng: random.Random) -> list[str]:
    """Deprecated float Abs/Max/Min over edge floats, including NaN order and signed zeros."""
    floats = ["00000000", "80000000", "00000001", "7f7fffff", "ff7fffff", "7f800000", "ff800000",
              "7fc00000", "ffc00000", "7fc00123", bits32(1.5), bits32(-2.25)]
    floats += [bits32(rng.uniform(-1e6, 1e6)) for _ in range(8)]
    requests = [f"{PREFIX}short_real abs {a} {a}" for a in floats]
    return requests + [f"{PREFIX}short_real {op} {a} {b}" for op in ("max", "min") for a in floats for b in floats]


def integer_requests(rng: random.Random) -> list[str]:
    """Integer helpers over boundaries, excluding the undefined overflow and division cases."""
    ints = [0, 1, -1, 2, -2, 3, -3, 7, -7, INT_MIN, INT_MAX, INT_MIN + 1, INT_MAX - 1, 46340, -46340]
    ints += [rng.randint(INT_MIN, INT_MAX) for _ in range(20)]
    requests = [f"{PREFIX}integer {op} {a} 0" for op in ("is_even", "is_odd") for a in ints]
    requests += [f"{PREFIX}integer abs {a} 0" for a in ints if a != INT_MIN]
    requests += [f"{PREFIX}integer square {a} 0" for a in ints if abs(a) <= 46340]
    pairs = [(a, b) for a in ints[:15] for b in ints[:15]]
    requests += [f"{PREFIX}integer {op} {a} {b}" for op in ("max", "min") for a, b in pairs]
    divisible = [(a, b) for a, b in pairs if b != 0 and (a, b) != (INT_MIN, -1)]
    return requests + [f"{PREFIX}integer modulus {a} {b}" for a, b in divisible]


def character_requests() -> list[str]:
    """Every byte 0..255 through the classifiers and conversions, plus equality pairs."""
    requests = [f"{PREFIX}character {byte:02x}" for byte in range(256)]
    requests += [f"{PREFIX}to_ext_character {byte:02x}" for byte in range(256)]
    units = [0, 0x41, 0x7F, 0x80, 0xFF, 0x100, 0x1FF, 0x20AC, 0xD800, 0xFFFF]
    requests += [f"{PREFIX}ext_character {unit:04x} {other:04x}" for unit in units for other in units[:4]]
    return requests + [f"{PREFIX}char_equal {a:02x} {b:02x}" for a in (0, 0x41, 0x80, 0xFF) for b in (0x41, 0xFF)]


def hash_requests(rng: random.Random) -> list[str]:
    """Hash engines over random byte strings of every length 0..64, typed values and seeds."""
    requests = []
    for length in range(65):
        data = bytes(rng.getrandbits(8) for _ in range(length)).hex() or "-"
        seed = rng.getrandbits(64)
        requests.append(f"{PREFIX}hash_bytes {seed:016x} {data}")
        requests.append(f"{PREFIX}hash_combine {seed:016x} {rng.randint(0, 64)} {bytes(rng.getrandbits(8) for _ in range(64)).hex()}")
    for text in (b"", b"a", b"ab", b"abc", b"abcd", b"\xff\x80", b"\xe9t\xe9", b"ab\0cd", b"hello world"):
        requests.append(f"{PREFIX}hash_bytes 0000000000000000 {text.hex() or '-'}")
    for name in HASH_TYPES:
        width = {"char": 1, "uint16": 2, "int": 4, "int64": 8, "double": 8}[name]
        values = [bytes(width), b"\xff" * width] + [bytes(rng.getrandbits(8) for _ in range(width)) for _ in range(6)]
        requests += [f"{PREFIX}hash_value {name} {value.hex()}" for value in values]
    return requests + [f"{PREFIX}hash_seeds"]


def c_string_requests() -> list[str]:
    """C-string equality with embedded NULs and ASCII Atof prefixes."""
    texts = [b"", b"a", b"abc", b"abc\0x", b"abd", b"\xff", b"\0"]
    requests = [f"{PREFIX}c_string_equal {a.hex() or '-'} {b.hex() or '-'}" for a in texts for b in texts]
    numbers = [b"", b"1", b" 2.5e1xyz", b"-0", b"inf", b"-nan", b"0x1p-2", b"1e400", b"1e-400", b".5", b"abc",
               b"4.9e-324", b"12\x0034"]
    return requests + [f"{PREFIX}atof {text.hex() or '-'}" for text in numbers]


def guid_texts(rng: random.Random) -> list[bytes]:
    """Valid, mixed-case, malformed, truncated, NUL-terminated and non-ASCII GUID strings."""
    texts = [b"00000000-0000-0000-0000-000000000000", b"12345678-1234-5678-9ABC-DEF012345678",
             b"ffffffff-ffff-ffff-ffff-ffffffffffff", b"0000000A-000B-000C-000D-00000000000E",
             b"12345678-1234-5678-9abc-def01234567", b"12345678-1234-5678-9abc-def0123456789",
             b"12345678x1234-5678-9abc-def012345678", b"1234567g-1234-5678-9abc-def012345678",
             b"12345678-1234-5678-9abc-def01234567\xe9", b"12345678-1234-5678-9abc-def012345678\0tail",
             b"12345678-1234-5678-9abc-def0123\0" + b"5678", b"", b"-" * 36, b" 2345678-1234-5678-9abc-def012345678"]
    for _ in range(12):
        texts.append(("%08x-%04x-%04x-%04x-%012x" % (rng.getrandbits(32), rng.getrandbits(16), rng.getrandbits(16),
                                                   rng.getrandbits(16), rng.getrandbits(48))).encode())
    return texts


def guid_requests(rng: random.Random) -> list[str]:
    """GUID constructors, conversions, comparisons, assignment and hashing."""
    texts = guid_texts(rng)
    requests = [f"{PREFIX}guid_text {text.hex() or '-'}" for text in texts]
    for text in texts:
        units = [byte | (rng.choice([0, 0x100, 0x2000]) if byte != 0 else 0) for byte in text.ljust(36, b"\0")[:40]]
        requests.append(f"{PREFIX}guid_utf16 {''.join(f'{unit:04x}' for unit in units)}")
    components = [[0] * 10, [-1, 65535, 65535, 65535, 255, 255, 255, 255, 255, 255], [INT_MIN, 1, 2, 0x0102, 3, 4, 5, 6, 7, 8]]
    components += [[rng.randint(INT_MIN, INT_MAX)] + [rng.getrandbits(16) for _ in range(3)] + [rng.getrandbits(8) for _ in range(6)]
                   for _ in range(8)]
    requests += [f"{PREFIX}guid_components {' '.join(map(str, c))}" for c in components]
    valid = [t for t in texts if GUID_FORMAT.fullmatch(t)]
    return requests + [f"{PREFIX}guid_compare {a.hex()} {b.hex()}" for a in valid[:6] for b in valid[:6]]


def misc_requests() -> list[str]:
    """Limits, version strings, time equality and single-threaded atomic sequences."""
    requests = [f"{PREFIX}limits", f"{PREFIX}version"]
    times = [0, -1, 2**63 - 1, -(2**63), 1_700_000_000]
    requests += [f"{PREFIX}time_equal {a} {b}" for a in times for b in times[:3]]
    for start in (0, 1, -1, INT_MAX, INT_MIN, 12345):
        requests.append(f"{PREFIX}atomic {start} {start} {start + 1 if start != INT_MAX else 0}")
    return requests


def append_standard_values_requests(requests: list[str]) -> None:
    """Append this area's requests with its own seed; append_standard_values_requests(requests)."""
    requests.extend(area_requests(random.Random(20260930)))


def area_requests(rng: random.Random) -> list[str]:
    """Every request of this area, drawn from `rng`; area_requests(random.Random(1))."""
    return (real_requests(rng) + short_real_requests(rng) + integer_requests(rng) + character_requests()
            + hash_requests(rng) + c_string_requests() + guid_requests(rng) + misc_requests())
