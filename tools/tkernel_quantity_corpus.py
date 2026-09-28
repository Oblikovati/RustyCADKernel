"""Request lines for the Quantity probe family (Color, ColorRGBA, Date and Period operators).

Covers every named color, every enum alias and name spelling rule, each color space's
validation limits, the HLS hue sectors and their edges (-1 grayscale, 360 wrap, NaN), the
sRGB and CIE Lab thresholds, hex and ARGB encodings, JSON round trips, and seeded random
colors. Floats travel as IEEE bit patterns so both sides read identical inputs. Inputs whose
native result is undefined behaviour (NaN or out-of-range float-to-int conversions in
ColorToHex, Color2argb and the hashes) are deliberately left out.
"""
from __future__ import annotations

import random
import re
import struct
from pathlib import Path

COLOR_COUNT = 509
ALIASES = ["BLUE1", "CHARTREUSE1", "CYAN1", "GOLD1", "GREEN1", "LIGHTCYAN1", "MAGENTA1",
           "ORANGE1", "ORANGERED1", "RED1", "TOMATO1", "YELLOW1"]
NAN = float("nan")
INF = float("inf")


def f(value: float) -> str:
    """IEEE single-precision bits of `value` as 8 hex digits.

    >>> f(1.0)
    '3f800000'
    """
    return struct.pack(">f", value).hex()


def d(value: float) -> str:
    """IEEE double-precision bits of `value` as 16 hex digits.

    >>> d(1.0)
    '3ff0000000000000'
    """
    return struct.pack(">d", value).hex()


def fs(*values: float) -> str:
    return " ".join(f(value) for value in values)


def ds(*values: float) -> str:
    return " ".join(d(value) for value in values)


def named_cases() -> list[str]:
    return [f"qc_named {index}" for index in range(COLOR_COUNT)]


def constructor_cases(rng: random.Random) -> list[str]:
    limits = {0: ([0.0, 1.0], [0.0, 1.0], [0.0, 1.0]), 1: ([0.0, 1.0], [0.0, 1.0], [0.0, 1.0]),
              2: ([-1.0, 0.0, 360.0], [0.0, 1.0], [0.0, 1.0]), 3: ([0.0, 100.0], [-100.0, 100.0], [-110.0, 100.0]),
              4: ([0.0, 100.0], [0.0, 135.0], [0.0, 360.0])}
    lines = []
    for space, ranges in limits.items():
        for axis in range(3):
            for edge in ranges[axis]:
                for nudge in [-1e-9, 0.0, 1e-9]:
                    values = [(r[0] + r[-1]) / 2 for r in ranges]
                    values[axis] = edge + nudge
                    lines.append(f"qc_ctor {space} {ds(*values)}")
        for _ in range(40):
            lines.append(f"qc_ctor {space} {ds(*(rng.uniform(r[0], r[-1]) for r in ranges))}")
    # HLS: negative hue is valid only with zero saturation; NaN components pass validation.
    lines += [f"qc_ctor 2 {ds(-5.0, 0.5, 0.0)}", f"qc_ctor 2 {ds(-5.0, 0.5, 0.5)}",
              f"qc_ctor 2 {ds(-1.0, 0.5, 0.5)}",
              f"qc_ctor 2 {ds(-30.0, 0.5, 0.0)}",
              f"qc_ctor 0 {ds(1.5, 0.0, 0.0)}", f"qc_ctor 1 {ds(0.0, -0.1, 0.0)}",
              f"qc_ctor 3 {ds(50.0, 100.0, -110.0)}", f"qc_ctor 3 {ds(100.0, -100.0, 100.0)}",
              f"qc_ctor 4 {ds(100.0, 135.0, 360.0)}", f"qc_ctor 4 {ds(0.0, 0.0, 0.0)}"]
    return lines


def vector_and_name_cases(rng: random.Random) -> list[str]:
    lines = [f"qc_vec3 {fs(*values)}" for values in
             [(0, 0, 0), (1, 1, 1), (0.5, 0.25, 0.125), (1.0000001, 0, 0), (0, -1e-30, 0), (0, 0, 2)]]
    lines += [f"qc_name {ds(*values)}" for values in
              [(0, 0, 0), (1, 1, 1), (1, 0, 0), (0.5, 0.5, 0.5), (1.1, 0, 0), (NAN, 0, 0), (0, 0, -0.0)]]
    for _ in range(200):
        values = [rng.random() for _ in range(3)]
        lines += [f"qc_vec3 {fs(*values)}", f"qc_name {ds(*values)}"]
    return lines


def color_names() -> list[str]:
    """Canonical names from the generated table (itself checked against Quantity_ColorTable.pxx)."""
    tables = sorted((Path(__file__).resolve().parents[1] / "crates/tkernel/src/quantity/color_names").glob("table_*.rs"),
                    key=lambda path: int(path.stem.split("_")[1]))
    return [name for path in tables for name in re.findall(r'"(\w+)"', path.read_text())]


def name_cases() -> list[str]:
    lines = []
    for name in color_names() + ALIASES:
        lines += [f"qc_from_name {name}", f"qc_from_name {name.lower()}", f"qc_from_name Quantity_NOC_{name}"]
    # A bare "QUANTITY_NOC_" makes the native SubString(14, 13) throw inside a noexcept function.
    for token in ["EMPTY", "QUANTITY_NOC_X", "Quantity_NOC_1", "UNDEFINED", "RE", "REDD", "xQUANTITY_NOC_RED",
                  "QUANTITY_NOC_QUANTITY_NOC_RED", "Red", "rEd1", "BLUE2", "GRAY", "GREY50", "é"]:
        lines.append(f"qc_from_name {token}")
    return lines


def hex_cases(rng: random.Random) -> list[str]:
    tokens = ["EMPTY", "#", "#F", "#FF", "#FF0", "FF0", "#FF0F", "FF0F", "#FF00", "#FFFF00", "FFFF00", "#ff8000",
              "#FFFF0080", "FFFF0080", "#FFFFFFFF", "#00000000", "#FFFFF", "#FFFFFFF", "#FFFFFFFFF", "#GG0000",
              "##FF0000", "#-FF000", "#+FF000", "0x0000", "#0x0000", "#abcdef", "#ABCDEF01", "123", "1234", "#1234"]
    for _ in range(60):
        digits = rng.choice([3, 4, 6, 8])
        tokens.append(("#" if rng.random() < 0.7 else "") + "".join(rng.choice("0123456789abcdefABCDEF") for _ in range(digits)))
    return [f"qc_hex {token}" for token in tokens]


def argb_cases(rng: random.Random) -> list[str]:
    values = [0, 0xFF0000, 0x00FF00, 0x0000FF, 0xFFFFFF, 0x7F7F7F, 0x010203, -1, -16777216, 2**31 - 1, -(2**31)]
    values += [rng.randrange(-(2**31), 2**31) for _ in range(80)]
    return [f"qc_argb {value}" for value in values]


def hls_cases(rng: random.Random) -> list[str]:
    hues = [-1.0, -0.5, -30.0, -59.99, -60.0, -61.0, 0.0, 30.0, 59.999, 60.0, 119.5, 120.0, 180.0, 240.0,
            299.9, 300.0, 359.99, 360.0, 360.01, 420.0, 1e10, -1e10, NAN, INF, -INF]
    lines = []
    for hue in hues:
        for light, saturation in [(0.5, 0.0), (0.5, 0.5), (1.0, 1.0), (0.0, 1.0), (0.25, 0.75)]:
            lines.append(f"qc_hls {fs(hue, light, saturation)}")
    for _ in range(100):
        lines.append(f"qc_hls {fs(rng.uniform(0, 360), rng.random(), rng.random())}")
    return lines


def srgb_hls_cases(rng: random.Random) -> list[str]:
    samples = [(0, 0, 0), (1, 1, 1), (0.5, 0.5, 0.5), (1, 0, 0), (0, 1, 0), (0, 0, 1), (1, 0, 0.0001),
               (0.2, 0.2, 0.1), (NAN, 0.5, 0.5), (0.5, NAN, 0.1), (-0.5, 0.2, 0.1), (2.0, 1.0, 0.0)]
    samples += [tuple(rng.random() for _ in range(3)) for _ in range(100)]
    return [f"qc_srgb_hls {fs(*values)}" for values in samples]


def lab_cases(rng: random.Random) -> list[str]:
    samples = [(0, 0, 0), (100, 0, 0), (50, 100, -110), (50, -100, 100), (0, 100, 100), (100, 100, 100),
               (8.0, 0, 0), (7.99, 0, 0), (50, 0.00001, 0.00001), (50, 1e-5, 0), (25, 60, 330),
               (1, 1, 1), (0.9, 0.2, 0.1), (NAN, 0, 0)]
    samples += [(rng.uniform(0, 100), rng.uniform(-100, 100), rng.uniform(-110, 100)) for _ in range(80)]
    samples += [(rng.random(), rng.random(), rng.random()) for _ in range(40)]
    return [f"qc_lab {fs(*values)}" for values in samples]


def gamma_cases(rng: random.Random) -> list[str]:
    edges = [0.0, -0.0, 1.0, 0.5, 0.0031308, 0.0031307, 0.0031309, 0.04045, 0.04044, 0.04046, 1e-10, 2.0,
             -0.25, 1e30, NAN, INF]
    lines = [f"qc_gamma {fs(v, v, 0.5)} {ds(v, v, 0.5)}" for v in edges]
    for _ in range(100):
        values = [rng.random() for _ in range(3)]
        lines.append(f"qc_gamma {fs(*values)} {ds(*values)}")
    return lines


def pair_cases(rng: random.Random) -> list[str]:
    fixed = [((1, 0, 0), (1, 0, 0)), ((1, 0, 0), (1, 0.0001, 0)), ((1, 0, 0), (1, 0.00011, 0)),
             ((0, 0, 0), (1, 1, 1)), ((0.5, 0.5, 0.5), (0.5, 0.5, 0.5)), ((0, 0, 1), (1, 1, 0)),
             ((0.2, 0.3, 0.4), (0.2, 0.3, 0.41)), ((0, 0, 0), (0, 0, 0.0))]
    lines = [f"qc_pair {ds(*a)} {ds(*b)}" for a, b in fixed]
    for _ in range(150):
        lines.append(f"qc_pair {ds(*(rng.random() for _ in range(6)))}")
    return lines


def change_cases(rng: random.Random) -> list[str]:
    colors = [(0, 0, 0), (1, 1, 1), (0.5, 0.5, 0.5), (1, 0, 0), (0.2, 0.4, 0.6), (0.9, 0.1, 0.3)]
    deltas = [0.0, 10.0, -10.0, 100.0, -100.0, 50.0, -150.0, 1e-3]
    lines = [f"qc_change {ds(*color)} {d(delta)}" for color in colors for delta in deltas]
    for _ in range(60):
        lines.append(f"qc_change {ds(*(rng.random() for _ in range(3)))} {d(rng.uniform(-120, 120))}")
    return lines


def epsilon_cases() -> list[str]:
    return [f"qc_epsilon {d(value)}" for value in [0.0001, 0.0, 0.001, 0.0010001, 1.0, 1e-12, 0.5]]


def json_cases() -> list[str]:
    texts = ['"RGB": [1, 0.5, 0]', '"RGB": [0, 0, 0]', '"RGB": [2, 0.5, 0]', '"RGB": [-1, 0, 0]', '"RGB": [1, 0.5]',
             '"RGB": [0.1, 0.2, 0.3, 0.4]', '"RGBA": [1, 0, 0]', '"rgb": [1, 0, 0]', '', '"RGB": [0.333333333333, 1e-3, 1]',
             '"RGB": [a, 0, 0]', '"RGB" : [1, 1, 1]', '{"RGB": [0.25, 0.5, 0.75]}']
    # Out-of-range RGBA values would reach the noexcept Quantity_ColorRGBA::SetValues and terminate.
    rgba = ['"RGBA": [1, 0.5, 0, 0.25]', '"RGBA": [0, 0, 0, 0]', '"RGBA": [1, 0, 0]',
            '"RGB": [1, 0, 0, 1]', '"RGBA": [0.5, 0.5, 0.5, 7]', '"RGBA": [0.1, 0.2, 0.3, -4]', '']
    return [f"qc_json {text}".rstrip() for text in texts] + [f"qrgba_json {text}".rstrip() for text in rgba]


def rgba_cases(rng: random.Random) -> list[str]:
    samples = [(0, 0, 0, 0), (1, 1, 1, 1), (1, 0, 0, 0.5), (0.5, 0.5, 0.5, 2.0), (1.5, 0, 0, 1), (0, 0, -0.5, 1),
               (0.2, 0.4, 0.6, 2.55), (0.2, 0.4, 0.6, 0.004)]
    samples += [(rng.random(), rng.random(), rng.random(), rng.random()) for _ in range(80)]
    lines = [f"qrgba_ctor {fs(*values)}" for values in samples]
    pairs = [((1, 0, 0, 1), (1, 0, 0, 1)), ((1, 0, 0, 1), (1, 0, 0, 0.9999)), ((1, 0, 0, 1), (1, 0, 0, 0.9998)),
             ((1, 0, 0, 1), (1, 0.001, 0, 1))]
    pairs += [(tuple(rng.random() for _ in range(4)), tuple(rng.random() for _ in range(4))) for _ in range(40)]
    lines += [f"qrgba_pair {fs(*a)} {fs(*b)}" for a, b in pairs]
    gammas = [(0, 0, 0, 0), (1, 1, 1, 1), (0.0031308, 0.04045, 0.5, -3.0), (2.0, -1.0, NAN, INF)]
    gammas += [tuple(rng.random() for _ in range(4)) for _ in range(40)]
    return lines + [f"qrgba_gamma {fs(*values)}" for values in gammas]


def time_cases(rng: random.Random) -> list[str]:
    lines = ["qperiod_ops 0 0 0 0", "qperiod_ops 1 0 0 999999", "qperiod_ops 0 999999 1 0", "qperiod_ops 5 5 5 5",
             "qperiod_ops 5 6 5 5", "qdate_ops 1 1 1979 0 0 0 0 0 0 0", "qdate_ops 1 1 1979 0 0 0 0 0 0 1",
             "qdate_ops 1 1 1979 0 0 0 0 1 0 1", "qdate_ops 12 31 2020 23 59 59 999 999 0 1"]
    for _ in range(60):
        lines.append(f"qperiod_ops {rng.randrange(10**6)} {rng.randrange(10**6)} {rng.randrange(10**6)} {rng.randrange(10**6)}")
        date = f"{rng.randint(1, 12)} {rng.randint(1, 28)} {rng.randint(1979, 2030)} {rng.randrange(24)} {rng.randrange(60)} {rng.randrange(60)} {rng.randrange(1000)} {rng.randrange(1000)}"
        lines.append(f"qdate_ops {date} {rng.randrange(10**8)} {rng.randrange(10**6)}")
    return lines


def cases() -> list[str]:
    """Every Quantity request line in a fixed, seeded order.

    >>> len(cases()) > 2000
    True
    """
    rng = random.Random(0x0C0102)
    lines = named_cases() + constructor_cases(rng) + vector_and_name_cases(rng) + name_cases()
    lines += hex_cases(rng) + argb_cases(rng) + hls_cases(rng) + srgb_hls_cases(rng) + lab_cases(rng)
    lines += gamma_cases(rng) + pair_cases(rng) + change_cases(rng) + epsilon_cases() + json_cases()
    lines += rgba_cases(rng) + time_cases(rng)
    return lines
