"""Differential corpus for the Message probe family, appended to verify_tkernel's corpus.

Requests are answered by `tools/oracle/tkernel_message_probe.cxx` and its Rust twin
`crates/tkernel/examples/oracle_probe/message/`. Inputs stay inside defined C++ behavior:
formatted numbers fit the native 64-byte buffers, and `Clear(type)` is only requested
when every top-level alert has that type (the native loop never ends otherwise).
"""
from __future__ import annotations

import random
import struct

import tkernel_message_report_corpus

PROBE_DIR = "target/oracle-tkernel/message-probe"


def hexb(data: bytes) -> str:
    """Byte pairs, `-` for none.

    >>> hexb(b"A"), hexb(b"")
    ('41', '-')
    """
    return data.hex() if data else "-"


def units(text: str | list[int]) -> str:
    """Comma-separated UTF-16 units, `-` for none.

    >>> units("Ab"), units([0x125, 100])
    ('41,62', '125,64')
    """
    values = list(text.encode("utf-16-le")) if isinstance(text, str) else None
    if values is not None:
        codes = [values[i] | values[i + 1] << 8 for i in range(0, len(values), 2)]
    else:
        codes = text
    return ",".join(f"{code:x}" for code in codes) if codes else "-"


def bits(value: float) -> str:
    """Hex bit pattern of a double.

    >>> bits(1.0)
    '3ff0000000000000'
    """
    return f"{struct.unpack('<Q', struct.pack('<d', value))[0]:x}"


INT_FORMATS = ["%d", "%5d", "%-5d|", "%+d", "% d", "%05d", "%x", "%#x", "%X", "%#X", "%o", "%#o",
               "%u", "%i", "%.3d", "%.0d", "%8.4d", "%-+6d", "%hd", "%hu", "%ld", "%lu", "%lx"]
REAL_FORMATS = ["%f", "%.2f", "%10.3f", "%-10.3f|", "%e", "%E", "%.0e", "%#.0e", "%g", "%G", "%#g",
                "%.10g", "%+.3e", "% f", "%#.0f", "%08.3f", "%.0f", "%g", "%.1g", "%-12.4G|", "%lf"]
TEXT_FORMATS = ["%s", "%10s", "%-10s|", "%.2s", "%5.1s", "%.0s", "%s"]
INTS = [0, 1, -1, 7, 42, -42, 255, 65535, 65536, -32769, 2147483647, -2147483648, 123456789]
SMALL_REALS = [0.0, -0.0, 1.5, -1.5, 2.5, 0.125, 1234567.891, 0.1, 999999.5, 9.9999995, 1e-5,
               123.456, -0.000123456, 1e20, 5e-324, 3.141592653589793, float("inf"), float("-inf"),
               float("nan"), 0.5, 1.0, 100.0, 1e15]
TEXTS = [b"", b"abc", b"hello world", "été".encode(), b"\xff\xfe", b"x" * 40]


def msg_steps(rng: random.Random, count: int) -> list[str]:
    """Random argument steps; each kind may or may not match a pending specification."""
    steps = []
    for _ in range(count):
        # After Get() the recorded positions are stale; a C format read there is garbage
        # (undefined behavior in Sprintf), so only extended-string arguments may follow.
        kind = rng.choice("eg" if "g" in steps else "siearg")
        if kind in "sa":
            steps.append(f"{kind}:{hexb(rng.choice(TEXTS))}")
        elif kind == "e":
            steps.append("e:" + units(rng.choice(["", "ext", "€5", "a b"])))
        elif kind == "i":
            steps.append(f"i:{rng.choice(INTS)}")
        elif kind == "r":
            steps.append(f"r:{bits(rng.choice(SMALL_REALS))}")
        else:
            steps.append("g")
    return steps


def msg_text(rng: random.Random) -> str:
    """A message mixing literal text and 1-4 specifications."""
    parts = []
    for _ in range(rng.randint(1, 4)):
        parts.append(rng.choice(["", "a", " = ", "Value ", "[", "] ", "x%%y "]))
        parts.append(rng.choice(INT_FORMATS + REAL_FORMATS + TEXT_FORMATS))
    parts.append(rng.choice(["", ".", " end", "%%"]))
    return "".join(parts)


def msg_cases(rng: random.Random) -> list[str]:
    """Message_Msg: fixed quirk texts, every single format, then random mixes."""
    quirks = ["100%%", "%%d", "%%%d", "%%s %s", "%", "abc%", "%h", "%l", "%5", "%-", "%q", "%hq",
              "%lld", "%hhd", "no format", "", "%d%d%d", "%s and %s", "%%", "%5.2", "a%%%"]
    cases = [f"msg_run {units(text)} i:5 r:{bits(2.5)} s:{hexb(b'txt')} g g" for text in quirks]
    cases += [f"msg_run {units([0x125, 0x64])} i:3 g", f"msg_run {units([0x25, 0x164, 0x20])} g i:1",
              f"msg_run {units([0x125, 0x73])} e:{units('ok')}", f"msg_run {units([0x25, 0x125, 0x64])} i:9 g"]
    for spec in INT_FORMATS:
        cases += [f"msg_run {units('<' + spec + '>')} i:{value} g" for value in INTS]
    for spec in REAL_FORMATS:
        cases += [f"msg_run {units('<' + spec + '>')} r:{bits(value)}" for value in SMALL_REALS]
    for spec in TEXT_FORMATS:
        cases += [f"msg_run {units('<' + spec + '>')} s:{hexb(text)}" for text in TEXTS]
        cases.append(f"msg_run {units('<' + spec + '>')} e:{units('wide')}")
    # Get() before the arguments leaves stale positions (the native quirk).
    cases += ["msg_run " + units("%-9.4f|%d|%s") + " i:4 g e:41 g",
              "msg_run " + units("%-9d%-9d") + " g g", "msg_run " + units("%s%s") + " g e:61 g"]
    for _ in range(400):
        cases.append(f"msg_run {units(msg_text(rng))} " + " ".join(msg_steps(rng, rng.randint(0, 6))))
    for text in [b"%d items", b"", "é %s".encode(), b"\xff %d", b"a%%b"]:
        cases.append(f"msg_set_c {hexb(text)}")
    return cases


def time_cases() -> list[str]:
    """FillTime over each branch and the %.2f rounding boundaries."""
    seconds = [0.0, 4.5, 26.3345, 34.496, 0.005, 0.015, 59.999, -1.25, 1e12, 0.125, 0.375]
    return [f"msg_fill_time {hour} {minute} {bits(second)}" for hour in [0, 1, 5, 23, 100, -1]
            for minute in [0, 6, 12, 59, -2] for second in seconds]


def metric_and_status_cases(rng: random.Random) -> list[str]:
    """Metric names and conversions; ExecStatus statics and flag algebra."""
    cases = ["msg_metrics"]
    names = ["NONE", "WallClock", "wallclock", "MemHeapUsage", "MemHeapUsage ", "", "ThreadCPUUserTime",
             "ProcessCPUSystemTime", "MemSwapUsagePeak", "Unknown"]
    cases += [f"msg_metric_from {hexb(name.encode())}" for name in names]
    statuses = [0, 0xFF] + [base + offset for base in (0x100, 0x200, 0x400, 0x800) for offset in range(32)]
    cases.append("msg_exec_static " + " ".join(map(str, statuses + [0x1FF, 0x300, 0x1000, 0x7FF, -1])))
    cases.append("msg_exec_by_index -3 132")
    ops = "DWAFdwafx"
    for _ in range(200):
        steps = []
        for _ in range(rng.randint(1, 10)):
            kind = rng.choice("sc|&+*" + ops)
            steps.append(kind + (str(rng.choice(statuses)) if kind in "sc|&+*" else ""))
        cases.append("msg_exec_ops " + " ".join(steps))
    return cases


MSGFILE_TEXTS = [
    ".{k}a\nFirst line\n  indented more\n\tafter tab\n.{k}b\n   Value B\n",
    "! comment\n.{k}a\r\nCRLF text\r\n  second\r\n\r\n.{k}b\r\nB\r\n",
    "text before keyword\n.{k}a\nA\n!inline comment\nstill A\n",
    ".  {k}a  \nspaced keyword\n.{k}a\nredefined\n",
    ".{k}a\n\n\nafter blank lines\n.{k}b\n",
    ".{k}a\nA\n.\nnever\n",
    ".{k}a\ncafé €\n.{k}b\nÿ raw\n",
    "   .{k}a\n        deep\n    shallow\n",
]


def msgfile_cases() -> list[str]:
    """Catalog loading from memory, files, directory lists and the environment."""
    cases = []
    for index, template in enumerate(MSGFILE_TEXTS):
        key = f"s{index}_"
        content = template.format(k=key).encode("utf-8")
        keys = [hexb(f"{key}{suffix}".encode()) for suffix in "abz"]
        cases.append(f"msgfile_string {hexb(content)} -1 " + " ".join(keys))
        hidden = content + b"\0." + key.encode() + b"c\nhidden"
        cases.append(f"msgfile_string {hexb(hidden)} -1 {hexb((key + 'c').encode())}")
        cases.append(f"msgfile_string {hexb(hidden)} {len(hidden)} {hexb((key + 'c').encode())}")
        key = f"f{index}_"
        content = template.format(k=key)
        path = hexb(f"{PROBE_DIR}/cat{index}.txt".encode())
        for encoding, bom in [("utf-8", b""), ("utf-16-le", b"\xff\xfe"), ("utf-16-be", b"\xfe\xff")]:
            data = bom + content.encode(encoding)
            cases.append(f"msgfile_file {path} {hexb(data)} " + " ".join(hexb(f"{key}{s}".encode()) for s in "ab"))
    return cases + msgfile_path_cases()


def path_hex(name: str) -> str:
    """Hex of a path inside the probe directory."""
    return hexb(f"{PROBE_DIR}/{name}".encode())


def msgfile_path_cases() -> list[str]:
    """Lengths, empty files, directory lists and environment lookups."""
    cases = [f"msgfile_string {hexb(b'.l0_a' + bytes([10]) + b'abc')} 4 {hexb(b'l0_a')}",
             f"msgfile_string {hexb(b'.l1_a' + bytes([10]) + b'abc')} 0 {hexb(b'l1_a')}",
             f"msgfile_file {path_hex('empty.txt')} - {hexb(b'none')}"]
    dirs = hexb(f"{PROBE_DIR}/d1 {PROBE_DIR}/missing".encode())
    # Files persist between the native and the Rust run: every file is written before use.
    cases.append(f"msgfile_file {path_hex('d1/dirs.txt')} {hexb(b'.d_a' + bytes([10]) + b'from d1')} {hexb(b'd_a')}")
    cases.append(f"msgfile_load {dirs} {hexb(b'dirs.txt')} {hexb(b'd_a')}")
    cases.append(f"msgfile_load {path_hex('d1')} {hexb(b'dirs.txt')} {hexb(b'd_a')}")
    cases.append(f"msgfile_file {path_hex('env/app.us')} {hexb(b'.e_a' + bytes([10]) + b'us text')} {hexb(b'e_a')}")
    cases.append(f"msgfile_file {path_hex('env/app.fr')} {hexb(b'.e_b' + bytes([10]) + b'fr text')} {hexb(b'e_b')}")
    cases.append(f"msgfile_env CSF_LANGUAGE - N {hexb(b'none')} {hexb(b'e_a')}")
    cases.append(f"msgfile_env MSGPROBE_DIR {path_hex('env')} N {hexb(b'app')} {hexb(b'e_a')}")
    cases.append(f"msgfile_env MSGPROBE_DIR {path_hex('env')} {hexb(b'fr')} {hexb(b'app')} {hexb(b'e_b')}")
    cases.append(f"msgfile_env MSGPROBE_DIR {path_hex('env/')} {hexb(b'.fr')} {hexb(b'app')} {hexb(b'e_b')}")
    cases.append(f"msgfile_env MSGPROBE_DIR - - {hexb(b'app')} {hexb(b'e_c')}")
    bad = b".Message_Msg_BadKeyword" + bytes([10]) + b"Missing: "
    cases.append(f"msgfile_string {hexb(bad)} -1 {hexb(b'q_unknown')}")
    return cases


def cases() -> list[str]:
    """Every Message request line in a fixed order.

    >>> isinstance(cases(), list)
    True
    """
    rng = random.Random(20260928)
    lines = msg_cases(rng) + time_cases() + metric_and_status_cases(rng) + msgfile_cases()
    return lines + tkernel_message_report_corpus.cases(rng)
