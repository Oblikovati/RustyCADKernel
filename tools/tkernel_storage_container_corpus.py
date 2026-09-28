"""Container contents for the Storage/FSD corpus: one random data set rendered as driver
write scripts, read scripts and the exact text, compatible-text and binary file bytes.
"""
from __future__ import annotations

import random
import struct

from tkernel_storage_encoding_corpus import bits, ext, fbits, hx


# ------------------------------------------------------------------ container contents

class Container:
    """One data set's sections, rendered as driver write scripts, read scripts and file bytes."""

    def __init__(self, rng: random.Random):
        def word() -> str:
            return "".join(rng.choice("abcXYZ_09") for _ in range(rng.randrange(1, 8)))
        self.count = rng.randrange(0, 5)
        self.version, self.date = "1.3", rng.choice(["09/28/2026", "", "01/02/1999"])
        self.schema, self.schema_version = rng.choice(["", "Schema", "PCDM"]), rng.choice(["", "7"])
        self.app_name = rng.choice(["", "App", "Ωmega", "a€b"])
        self.app_version = rng.choice(["", "1.0", "v2"])
        self.data_type = rng.choice(["", "MDTV-Standard", "é"])
        self.info = [word() for _ in range(rng.randrange(0, 3))]
        self.comments = [rng.choice(["note", "", "Ünïcödé", "x y"])
                         for _ in range(rng.randrange(0, 3))]
        self.types = [(index + 1, word()) for index in range(rng.randrange(0, 3))]
        self.roots = [(word(), rng.randrange(1, 5), word()) for _ in range(rng.randrange(0, 3))]
        self.refs = [(rng.randrange(1, 9), rng.randrange(1, 4)) for _ in range(rng.randrange(0, 3))]
        self.objects = [(rng.randrange(1, 9), rng.randrange(1, 4), scalars(rng))
                        for _ in range(rng.randrange(0, 3))]

    def info_argument(self) -> str:
        fields = [str(self.count), hx(self.version.encode()), hx(self.date.encode()),
                  hx(self.schema.encode()), hx(self.schema_version.encode()), ext(self.app_name),
                  hx(self.app_version.encode()), ext(self.data_type),
                  ";".join(hx(line.encode()) for line in self.info) or "-"]
        return ",".join(fields)

    def write_script(self) -> list[str]:
        ops = ["ow", "bwi", "wi=" + self.info_argument(), "ewi", "bwc",
               "wc=" + (";".join(ext(c) for c in self.comments) or "-"), "ewc", "bwt",
               f"sts={len(self.types)}"] + [f"wt={n},{hx(name.encode())}" for n, name in self.types]
        ops += ["ewt", "bwr", f"ssr={len(self.roots)}"]
        ops += [f"wr={hx(name.encode())},{ref},{hx(kind.encode())}" for name, ref, kind in self.roots]
        ops += ["ewr", "bwf", f"ssf={len(self.refs)}"] + [f"wf={r},{t}" for r, t in self.refs]
        ops += ["ewf", "bwd", "tl"]
        for ref, kind, values in self.objects:
            ops += [f"woh={ref},{kind}", "bwp"] + [put_op(k, v) for k, v in values] + ["bwo", "ewo", "ewp"]
        return ops + ["ewd", "ie", "cl"]

    def read_script(self, with_types: bool = True) -> list[str]:
        ops = ["or", "bri", "ri", "eri", "brc", "rc", "erc", "brt", "ts"]
        ops += ["rt"] * (len(self.types) if with_types else 0)
        ops += ["ert", "brr", "rs"] + ["rr"] * len(self.roots) + ["err", "brf", "fs"]
        ops += ["rf"] * len(self.refs) + ["erf", "brd"]
        get = {"pi": "gi", "pr": "gr", "pb": "gb", "pd": "gd", "pc": "gc", "pe": "ge", "pf": "gf"}
        for _, _, values in self.objects:
            ops += ["roh", "brp"] + [get[k] for k, _ in values] + ["bro", "ero", "erp"]
        return ops + ["erd", "ie", "tl", "cl"]


def scalars(rng: random.Random) -> list[tuple[str, object]]:
    makers = {"pi": lambda: rng.randrange(-100000, 100000), "pr": lambda: rng.randrange(0, 50),
              "pb": lambda: rng.randrange(2), "pc": lambda: rng.randrange(256),
              "pe": lambda: rng.randrange(65536),
              "pd": lambda: rng.choice([0.1, -2.5, 1e300, 3.0, rng.uniform(-1e3, 1e3)]),
              "pf": lambda: rng.choice([0.5, -1.25, 3.4e38, rng.uniform(-10, 10)])}
    kinds = list(makers)
    values = []
    for _ in range(rng.randrange(0, 5)):
        kind = rng.choice(kinds)
        values.append((kind, makers[kind]()))
    return values


def put_op(kind: str, value: object) -> str:
    if kind == "pd":
        return f"pd={bits(value)}"
    if kind == "pf":
        return f"pf={fbits(value)}"
    return f"{kind}={value}"


def text_scalar(kind: str, value: object) -> bytes:
    if kind == "pd":
        return b"%.17g " % value
    if kind == "pf":
        return b"%.17g " % struct.unpack(">f", struct.pack(">f", value))[0]
    if kind == "pc":
        return b"%d " % (value if value < 128 else 65536 + value - 256)
    if kind == "pe":
        return b"%d " % (value if value < 32768 else value - 65536)
    return b"%d " % value


def utf16_units(text: str) -> list[int]:
    data = text.encode("utf-16-be")
    return list(struct.unpack(f">{len(data) // 2}H", data))


def text_ext_line(text: str, compatible: bool) -> bytes:
    units = utf16_units(text)
    if compatible:
        return b"%d " % len(units) + b"".join(text_scalar("pe", u) for u in units) + b"\n"
    return b"".join(struct.pack(">H", u) for u in units) + b"\0\n"


def text_file(c: Container, compatible: bool) -> bytes:
    lines = [c.version, c.date, c.schema, c.schema_version]
    out = (b"CMPFILE\n" if compatible else b"FSDFILE\n") + b"BEGIN_INFO_SECTION\n"
    out += b"%d\n" % c.count + b"".join(s.encode() + b"\n" for s in lines)
    out += text_ext_line(c.app_name, compatible) + c.app_version.encode() + b"\n"
    out += text_ext_line(c.data_type, compatible) + b"%d\n" % len(c.info)
    out += b"".join(i.encode() + b"\n" for i in c.info)
    out += b"END_INFO_SECTION\nBEGIN_COMMENT_SECTION\n%d\n" % len(c.comments)
    out += b"".join(text_ext_line(x, compatible) for x in c.comments) + b"END_COMMENT_SECTION\n"
    out += b"BEGIN_TYPE_SECTION\n%d\n" % len(c.types)
    out += b"".join(b"%d %s\n" % (n, t.encode()) for n, t in c.types)
    out += b"END_TYPE_SECTION\nBEGIN_ROOT_SECTION\n%d\n" % len(c.roots)
    out += b"".join(b"%d %s %s\n" % (r, n.encode(), k.encode()) for n, r, k in c.roots)
    out += b"END_ROOT_SECTION\nBEGIN_REF_SECTION\n%d\n" % len(c.refs)
    out += b"".join(b"%d %d\n" % p for p in c.refs) + b"END_REF_SECTION\nBEGIN_DATA_SECTION"
    for ref, kind, values in c.objects:
        body = b"".join(text_scalar(k, v) for k, v in values)
        if compatible:
            out += b"\n#%d%%%d " % (ref, kind) + body
        else:
            out += b"\n#%d=%%%d( " % (ref, kind) + body + b"( ) )"
    return out + b"\nEND_DATA_SECTION\n"


def binary_scalar(kind: str, value: object) -> bytes:
    formats = {"pi": "<i", "pr": "<i", "pb": "<i", "pd": "<d", "pf": "<f", "pc": "<B", "pe": "<H"}
    return struct.pack(formats[kind], value)


def binary_string(text: str) -> bytes:
    data = text.encode()
    return struct.pack("<i", len(data)) + data


def binary_ext(text: str) -> bytes:
    data = text.encode("utf-16-le")
    return struct.pack("<i", len(data) // 2) + data


def binary_file(c: Container) -> bytes:
    body = struct.pack("<i", c.count)
    body += b"".join(binary_string(s) for s in [c.version, c.date, c.schema, c.schema_version])
    body += binary_ext(c.app_name) + binary_string(c.app_version) + binary_ext(c.data_type)
    body += struct.pack("<i", len(c.info)) + b"".join(binary_string(i) for i in c.info)
    sections = [
        struct.pack("<i", len(c.comments)) + b"".join(binary_ext(x) for x in c.comments),
        struct.pack("<i", len(c.types)) + b"".join(struct.pack("<i", n) + binary_string(t) for n, t in c.types),
        struct.pack("<i", len(c.roots)) + b"".join(struct.pack("<i", r) + binary_string(n) + binary_string(k)
                                                    for n, r, k in c.roots),
        struct.pack("<i", len(c.refs)) + b"".join(struct.pack("<ii", *p) for p in c.refs),
        b"".join(struct.pack("<ii", ref, kind) + b"".join(binary_scalar(k, v) for k, v in values)
                 for ref, kind, values in c.objects),
    ]
    position = 7 + 52 + len(body)
    offsets = [7, position]
    for section in sections:
        offsets += [position, position + len(section)]
        position += len(section)
    return b"BINFILE" + struct.pack("<13i", 0x04030201, *offsets) + body + b"".join(sections)
