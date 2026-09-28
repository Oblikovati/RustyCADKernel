"""Differential corpus for the Storage/FSD probe family, appended to verify_tkernel's corpus.

Requests are answered by tools/oracle/tkernel_storage_probe.cxx and its Rust twin
crates/tkernel/examples/oracle_probe/storage/. The cases avoid inputs on which the native
code has undefined behaviour or never returns: text-driver tag searches, word reads and
extended-line reads in a stream that failed without reaching end of file, extraction of a
TCollection_AsciiString at end of file, binary-driver calls on a null FILE, Base64 bytes
of 0x80 and above, and ReadCompleteInfo on offsets that match no section.
"""
from __future__ import annotations

import random
import struct

from tkernel_storage_container_corpus import Container, binary_file, put_op, scalars, text_file
from tkernel_storage_encoding_corpus import bits, ext, fbits, hx

# ------------------------------------------------------------------- number formatting

SPECIAL_DOUBLES = [
    0.0, -0.0, 1.0, -1.0, 0.1, 0.5, 2.5, 0.125, 1.0 / 3.0, 2.0 / 3.0, 123456.0, 1234567.0,
    1e-5, 1e-4, 9.9999e-5, 0.0001, 1e15, 1e16, 1e17, 1e21, 1e22, 1e100, 1e-300,
    5e-324, 2.2250738585072014e-308, 1.7976931348623157e308, 8796093022208.03125,
    8796093022208.09375, 123456789012.5, 0.15, 0.35, 100.0, 1e6, 999999.5, 9999995.0,
]




def format_cases(rng: random.Random) -> list[str]:
    lines = []
    specials = [bits(v) for v in SPECIAL_DOUBLES] + [
        "7ff0000000000000", "fff0000000000000", "7ff8000000000000", "fff8000000000000",
        "7ff0000000000001", "0000000000000001", "8000000000000001"]
    for precision in [0, 1, 2, 3, 6, 15, 16, 17, 18, 20]:
        for value in specials:
            lines.append(f"stor_fmt {precision} {value}")
    for _ in range(300):
        value = rng.choice([rng.getrandbits(64), struct.unpack(">Q", struct.pack(">d", rng.uniform(-1e6, 1e6)))[0],
                            struct.unpack(">Q", struct.pack(">d", rng.randrange(-10**9, 10**9) / 1000))[0]])
        lines.append(f"stor_fmt {rng.choice([6, 17])} {value:016x}")
    for _ in range(100):
        lines.append(f"stor_fmtf 17 {rng.getrandbits(32):08x}")
    for value in [0.1, 1.5, 3.4e38, 1e-45, 16777217.0]:
        lines.append(f"stor_fmtf 17 {fbits(value)}")
    return lines


# ---------------------------------------------------------------------- stream extraction

STREAM_TEXTS = [
    b"", b" ", b"42", b" 42 ", b"-7", b"+7", b"+", b"-", b"x", b"0x1f", b"007", b"-0",
    b"2147483647", b"2147483648", b"-2147483648", b"-2147483649", b"99999999999999999999",
    b"-99999999999999999999", b"9223372036854775807", b"9223372036854775808",
    b"32767", b"32768", b"-32768", b"-32769", b"65535", b"65536", b"-1", b"-65535", b"-65536",
    b"0", b"1", b"2", b"\t\n 5\r", b"1 2 3", b"abc def\nghi\n", b"line\n\nnext", b"a\0b c",
    b"1.5e3", b"12x", b"\n\n\n", b" \x0b\x0c7", b"\xb5\xff 3",
]
STREAM_OPS = ["g", "i", "s", "u", "b", "w", "tg", "tp", "c", "l1", "l2", "l3", "l8", "l100",
              "r1", "r2", "r5", "x2", "x0", "l0"]


def stream_cases(rng: random.Random) -> list[str]:
    lines = []
    for text in STREAM_TEXTS:
        for op in ["i", "s", "u", "b", "w", "g", "l8", "r3"]:
            lines.append(f"stor_stream {hx(text)} {op} {op} tg tp g")
    long_word = b"1" * 150 + b" 2"
    lines += [f"stor_stream {hx(long_word)} w w w g", f"stor_stream {hx(b'9' * 99)} w g tg",
              f"stor_stream {hx(b'8' * 99 + b' ')} w g", f"stor_stream {hx(b'7' * 98)} w g",
              f"stor_stream {hx(b'x' * 20)} x5 w w", f"stor_stream {hx(b'a' * 10 + b'\n')} l5 c l20 g"]
    alphabet = b"0123456789-+ \n\t x.e\r\0"
    for _ in range(400):
        text = bytes(rng.choice(alphabet) for _ in range(rng.randrange(0, 16)))
        ops = " ".join(rng.choice(STREAM_OPS) for _ in range(rng.randrange(1, 9)))
        lines.append(f"stor_stream {hx(text)} {ops}")
    return lines


# ----------------------------------------------------------------------------- Base64

def base64_cases(rng: random.Random) -> list[str]:
    lines = []
    for length in range(0, 12):
        data = bytes(rng.randrange(256) for _ in range(length))
        lines.append(f"stor_b64e {hx(data)}")
        encoded = __import__("base64").b64encode(data)
        lines.append(f"stor_b64d {hx(encoded)}")
        for capacity in [-1, 0, len(encoded) - 1, len(encoded), len(encoded) + 3]:
            lines.append(f"stor_b64ec {capacity} {hx(data)}")
            lines.append(f"stor_b64dc {capacity} {hx(encoded)}")
    alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=-_,.*?!@ \n"
    for _ in range(300):
        text = bytes(rng.choice(alphabet) for _ in range(rng.randrange(0, 14)))
        lines.append(f"stor_b64d {hx(text)}")
        lines.append(f"stor_b64dc {rng.randrange(-1, 12)} {hx(text)}")
    for _ in range(100):
        data = bytes(rng.randrange(256) for _ in range(rng.randrange(0, 40)))
        lines.append(f"stor_b64e {hx(data)}")
    return lines


# ---------------------------------------------------------------- misc Storage entries

MIGRATED = ["TDataStd_Shape", "TDataStd_Constraint", "TDataStd_Geometry", "TDataStd_Axis",
            "TDataStd_Point", "TDataStd_Plane", "TDataStd_Position", "TDataStd_Placement",
            "TDataStd_PatternStd", "TPrsStd_AISPresentation", "PDataStd_Shape",
            "PDataStd_Constraint", "PDataStd_Geometry", "PDataStd_Axis", "PDataStd_Point",
            "PDataStd_Plane", "PDataStd_Position", "PDataStd_Placement", "PDataStd_PatternStd"]


def misc_cases() -> list[str]:
    lines = ["stor_ver"]
    for name in MIGRATED + ["TDataStd_Real", "", "tdatastd_shape", "TDataXtd_Shape"]:
        lines.append(f"stor_migr {hx(name.encode())}")
    for text in [b"", b"B", b"BINFILE", b"BINFILEX", b"FSDFILE\n", b"CMP", b"\0\0\0\0\0\0\0",
                 b"AB\0CDEFG", b"ABCDEF"]:
        lines.append(f"stor_magic {hx(text)}")
    return lines


# ---------------------------------------------------------------------- driver scripts

def file_argument(data: bytes | None) -> str:
    """'-' for no file, '@' for an empty file, else the file's bytes in hex."""
    if data is None:
        return "-"
    return hx(data) if data else "@"


def drv(kind: str, data: bytes | None, ops: list[str]) -> str:
    return f"stor_drv {kind} {file_argument(data)} " + " ".join(ops)


def truncations(rng: random.Random, data: bytes, count: int) -> list[bytes]:
    return [data[:rng.randrange(0, len(data))] for _ in range(count)] if data else []


def container_cases(rng: random.Random) -> list[str]:
    lines = []
    for _ in range(70):
        box = Container(rng)
        files = {"f": text_file(box, False), "c": text_file(box, True), "b": binary_file(box)}
        for kind, data in files.items():
            lines.append(drv(kind, None, box.write_script()))
            lines.append(drv(kind, data, box.read_script()))
            lines.append(drv(kind, data, ["or", "hr", "rdr", "tdr", "ie", "tl"]))
            text = kind != "b"
            for cut in truncations(rng, data, 3):
                lines.append(drv(kind, cut, box.read_script(with_types=not text)))
                lines.append(drv(kind, cut, ["or", "hr", "rdr"] + ([] if text else ["tdr"])))
        lines.append(drv("c", files["c"], ["or", "bri", "ri", "ri", "ie"]))
    return lines


TEXT_TOKENS = [b"42", b"-7", b"0", b"1", b"2", b"-1", b"1.5", b"0.10000000000000001", b"1e400",
               b"-1e-400", b"nan", b"inf", b"-inf", b"0x10", b"1.5x", b"abc", b"65456", b"-80",
               b"70000", b"-32769", b"32767", b"2147483648", b"3.4028234663852886e+38", b"1e39",
               b"1" * 150, b"+5", b"-", b".5", b"5.", b"1e", b"\t", b"\n", b"\r\n"]
TEXT_GETS = ["gi", "gr", "gc", "ge", "gb", "gd", "gf"]


def scalar_cases(rng: random.Random) -> list[str]:
    lines = []
    for _ in range(250):
        tokens = [rng.choice(TEXT_TOKENS) for _ in range(rng.randrange(0, 8))]
        text = b" ".join(tokens) + rng.choice([b"", b" ", b"\n"])
        gets = [rng.choice(TEXT_GETS) for _ in range(rng.randrange(1, 9))]
        lines.append(drv(rng.choice("fc"), text, ["or"] + gets + ["ie", "tl"]))
    for _ in range(150):
        data = bytes(rng.randrange(256) for _ in range(rng.randrange(0, 24)))
        gets = [rng.choice(TEXT_GETS) for _ in range(rng.randrange(1, 8))]
        lines.append(drv("b", data, ["or"] + gets + ["ie", "tl"]))
    for _ in range(120):
        box_values = scalars(rng) or [("pi", 1)]
        for kind in "fcb":
            lines.append(drv(kind, None, ["ow"] + [put_op(k, v) for k, v in box_values] + ["tl", "cl"]))
    return lines


OBJECT_TEXTS = [b"#1=%2( 5 ) )\n", b"  #1 = % 2 ( 5 ) )  \n", b"#x", b"\n#1=%2", b"#1%2 7 \r\n",
                b"#1%2 7 \n", b"#-3=%-4(  ) )\n", b"#1=%2)", b"#1 %2 \r", b"", b" ", b"#1=%2(( ) )\r\n"]


def mode_cases() -> list[str]:
    body = b"FSDFILE\nBEGIN_INFO_SECTION\n0\n\n\n\n\n\0\n\n\0\n0\nEND_INFO_SECTION\n"
    lines = []
    for kind in "fc":
        lines += [drv(kind, None, ["pi=5", "bwi", "gi", "ie", "tl", "bri", "cl"]),
                  drv(kind, None, ["on", "tl", "pi=3", "ie", "cl", "cl"]),
                  drv(kind, None, ["or", "or", "cl"]),
                  drv(kind, body, ["or", "or", "pi=5", "bwi", "tl", "cl", "or", "sk", "gi", "cl"]),
                  drv(kind, body, ["ou", "pi=9", "pd=3ff8000000000000", "tl", "cl"]),
                  drv(kind, None, ["ou", "cl"]),
                  drv(kind, body, ["ou", "bri", "tl", "cl"]),
                  drv(kind, body, ["or", "sk", "sk", "sk", "gi", "ie", "tl"])]
        for text in OBJECT_TEXTS:
            lines.append(drv(kind, text, ["or", "roh", "brp", "gi", "bro", "ero", "erp", "ie"]))
            lines.append(drv(kind, text, ["or", "erp", "ie"]))
    lines += [drv("b", None, ["ow", "tl", "ie", "pi=3", "cl", "cl"]),
              drv("b", None, ["on"]), drv("b", None, ["or"]),
              drv("b", body, ["or", "or", "pi=5", "bwi", "cl"]),
              drv("b", body, ["ou", "pi=7", "tl", "cl"]),
              drv("b", None, ["ow", "bwi", "ewi", "bwc", "ewc", "bwt", "ewt", "bwr", "ewr", "bwf", "ewf",
                              "bwd", "ewd", "cl"])]
    for kind in "fcb":
        for data in [None, b"", b"FSDFILE", b"FSDFILE\n", b"CMPFILE", b"BINFILE", b"BINF", b"FSDFIL",
                     b"\0SDFILE"]:
            lines.append(f"stor_good {kind} {file_argument(data)}")
    return lines


# ---------------------------------------------------------------- data and schema scripts

NAMES = [b"a", b"b", b"doc", b"1", b"2", b"root", b"x y"[:1]]


def data_cases(rng: random.Random) -> list[str]:
    lines = []
    for _ in range(150):
        ops = []
        for _ in range(rng.randrange(1, 14)):
            name = hx(rng.choice(NAMES))
            ops.append(rng.choice([f"ar={name}", "au", f"ao={name}", f"rm={name}", f"fd={name}",
                                   f"ir={name}", f"ur={name}", f"at={name},{rng.randrange(-1, 5)}",
                                   f"tn={rng.randrange(-1, 5)}", f"tt={name}", f"it={name}", f"ai={name}",
                                   f"ac={ext('c')}", "cl", "ce", "st", "dump"]))
        lines.append("stor_data " + " ".join(ops + ["dump"]))
    return lines


def bucket_cases(rng: random.Random) -> list[str]:
    lines = []
    for _ in range(120):
        size, length, ops = rng.randrange(1, 5), 0, []
        for _ in range(rng.randrange(1, 20)):
            choice = rng.random()
            if choice < 0.5:
                ops.append(rng.choice("aan"))
                length += 1
            elif choice < 0.65 and length:
                ops.append(f"v{rng.randrange(1, length + 1)}")
            elif choice < 0.75:
                ops.append("len")
            elif choice < 0.8:
                ops.append("clr")
                length = 0
            elif length:
                ops.append("it")
        lines.append(f"stor_bucket {size} " + " ".join(ops))
    return lines


def callback_cases(rng: random.Random) -> list[str]:
    lines = []
    for _ in range(80):
        ops = []
        for _ in range(rng.randrange(1, 12)):
            name = hx(rng.choice(NAMES + [b"PColStd", b"TDataStd_Real"]))
            ops.append(rng.choice([f"add={name}", f"addn={name}", f"rm={name}", "ls", "clr", "use",
                                   "dont", "is", "rst", "setn", "dflt"]))
        lines.append("stor_cb " + " ".join(ops + ["ls", "is", "dflt"]))
    return lines


def schema_cases(rng: random.Random) -> list[str]:
    lines = []
    for _ in range(60):
        config = []
        for _ in range(rng.randrange(0, 8)):
            name = hx(rng.choice(NAMES))
            config.append(rng.choice([f"app={ext('App')}", f"ver={hx(b'1.0')}", f"type={ext('Std')}",
                                      f"info={name}", f"com={ext('note')}", f"root={name}", "aroot",
                                      f"sname={hx(b'Schema')}", f"sver={hx(b'2')}"]))
        if rng.random() < 0.15:
            config.append(f"oroot={hx(b'obj')}")
        for kind in "fcb":
            lines.append(f"stor_schema {kind} {rng.choice('wwwurn')} " + " ".join(config))
    return lines


def complete_cases(rng: random.Random) -> list[str]:
    lines = []
    for _ in range(60):
        box = Container(rng)
        box.refs, box.objects = [], []
        box.roots = [(name, rng.randrange(1, box.count + 2), kind) for name, _, kind in box.roots]
        data = binary_file(box)
        lines.append(f"stor_complete {hx(data)}")
        for cut in truncations(rng, data, 2):
            lines.append(f"stor_complete {hx(cut)}")
    return lines


def bstream_cases(rng: random.Random) -> list[str]:
    lines = []
    for _ in range(60):
        box = Container(rng)
        count = str(rng.randrange(2))
        header = ",".join(str(rng.randrange(-5, 500)) for _ in range(13))
        ops = [rng.choice([f"pi={rng.randrange(-9, 9)},{count}", f"wh={header},{count}",
                           f"wi={box.info_argument()},{count}",
                           f"wc={';'.join(ext(c) for c in box.comments) or '-'},{count}", "tp"])
               for _ in range(rng.randrange(1, 6))]
        lines.append("stor_bstream - " + " ".join(ops + ["tp"]))
        box.objects = []
        data = binary_file(box)[7:]
        reads = ["rh", "rhd", "rc", "ts"] + ["rt"] * len(box.types) + ["ts"] + ["rr"] * len(box.roots)
        reads += ["ts"] + ["rf"] * len(box.refs) + ["tg", "gi", "tg"]
        lines.append(f"stor_bstream {hx(data)} " + " ".join(reads))
        for cut in truncations(rng, data, 2):
            lines.append(f"stor_bstream {hx(cut)} " + " ".join(reads))
        lines.append(f"stor_bstream {hx(bytes(rng.randrange(256) for _ in range(10)))} gi rs re gi tg")
    return lines


def cases() -> list[str]:
    """Every storage request line, in a fixed order from a fixed seed.

    >>> len(cases()) > 1000
    True
    """
    rng = random.Random(1996)
    return (format_cases(rng) + stream_cases(rng) + base64_cases(rng) + misc_cases()
            + container_cases(rng) + scalar_cases(rng) + mode_cases() + data_cases(rng)
            + bucket_cases(rng) + callback_cases(rng) + schema_cases(rng) + complete_cases(rng)
            + bstream_cases(rng))
