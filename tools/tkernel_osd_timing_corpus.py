"""Request lines for the OSD timing probe family (OSD_Timer, OSD_Chronometer, OSD statics).

Covers Timer::Reset(double) on normal, fractional, boundary (minute/hour edges, the int
range of the native truncation), negative and non-finite elapsed times; stopped
chronometers in both CPU scopes; RealToCString on special, subnormal, extreme, integral
and seeded random doubles; CStringToReal on well-formed, partial, empty, whitespace,
special and round-tripped texts; the signal statics (mode, floating-point traps, stack
trace length) and the SIGINT -> ControlBreak path.
"""
from __future__ import annotations

import random
import struct


def bits(value: float) -> str:
    """IEEE-754 bit pattern of `value` as 16 hex digits.

    >>> bits(1.0)
    '3ff0000000000000'
    """
    return f"{struct.unpack('<Q', struct.pack('<d', value))[0]:016x}"


def hex_text(text: str) -> str:
    """Hex-encoded ASCII bytes, `-` for the empty text.

    >>> hex_text("1.5")
    '312e35'
    """
    return text.encode().hex() or "-"


NAN_NEG = "fff8000000000000"
TIMER_VALUES = [0.0, -0.0, 5.5, 59.999, 60.0, 61.25, 3599.5, 3600.0, 3725.5, 86399.999999,
                1e6 + 0.25, 1e-9, 0.1, -5.5, -3725.5, 2147483647.5, 2147483648.0,
                -2147483648.0, -2147483649.5, 1e20, -1e20, 4e9, float("inf"),
                float("-inf"), float("nan")]
REAL_VALUES = [0.0, -0.0, 1.0, -1.0, 1.5, 0.1, 0.2, 1 / 3, 2 / 3, 10.0, 100.0, 1e5, 1e-5,
               123456789.0, 1e22, 1e23, 1e100, 1e-100, 1e300, 1e-300, 5e-324, 2.5e-320,
               2.2250738585072014e-308, 2.2250738585072009e-308, 1.7976931348623157e308,
               -1.7976931348623157e308, 9.999999999999999e22, 0.5, 0.25, 0.125,
               3.141592653589793, 2.718281828459045, float("inf"), float("-inf"),
               float("nan"), 1e15, 1e16, 1e17, 4503599627370496.0, 9007199254740993.0]
TEXTS = ["1.5", "", " 1.5", "1.5 ", "\t-2.25", "1e5", "1E5", "1e400", "-1e400", "1e-400",
         "5e-324", "2e-324", "inf", "-inf", "INF", "infinity", "nan", "NaN", "nan(123)",
         "0x1p3", "0X1.8p1", "1,5", "abc", "1.5e", "1.5e+", "+.5", ".", "-", "+", "1.",
         "00012", "1e0000005", "123456789012345678901234567890", "0.000000000000000000001",
         "1.5\n", "3.14159265358979323846264338327950288"]


def cases() -> list[str]:
    """Every timing request line, in a fixed order.

    >>> any(c.startswith("osd_ctrl_break") for c in cases())
    True
    """
    rng = random.Random(1992)
    lines = [f"osd_timer_reset {bits(value)}" for value in TIMER_VALUES]
    lines.append(f"osd_timer_reset {NAN_NEG}")
    lines += [f"osd_timer_reset {bits(rng.uniform(0, 400000))}" for _ in range(40)]
    lines += ["osd_chrono 0", "osd_chrono 1"]
    reals = REAL_VALUES + [rng.uniform(-1e6, 1e6) for _ in range(60)]
    reals += [struct.unpack("<d", struct.pack("<Q", rng.getrandbits(64)))[0] for _ in range(200)]
    lines += [f"osd_real_to_cstring {bits(value)}" for value in reals]
    lines.append(f"osd_real_to_cstring {NAN_NEG}")
    texts = TEXTS + [repr(value) for value in REAL_VALUES if value == value]
    texts += [f"{rng.uniform(-1e3, 1e3):.{rng.randrange(1, 20)}e}" for _ in range(40)]
    lines += [f"osd_cstring_to_real {hex_text(text)}" for text in texts]
    lines += ["osd_signal_stack 10", "osd_signal_stack -3", "osd_signal_stack 0"]
    for mode in range(4):
        for floating in range(2):
            if mode in (1, 2) and floating:
                continue  # never leave FP traps armed with native handlers installed
            lines.append(f"osd_thread_signal {mode} {floating}")
    lines += ["osd_signal 0 0", "osd_signal 0 1", "osd_floating 1", "osd_floating 0"]
    lines += ["osd_ctrl_break", "osd_sleep 0 0", "osd_signal 0 0"]
    return lines
