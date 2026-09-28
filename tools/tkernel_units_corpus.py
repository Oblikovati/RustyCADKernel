"""Request lines for the Units / UnitsAPI / UnitsMethods probe family.

The native probe keeps the package statics for the whole run, so the order of the lines
is part of the corpus: the empty-cache quirks (`ToSI("")`, `FirstQuantity("")`) come
first. Every string argument is hex-encoded ("-" is empty); see
tools/oracle/tkernel_units_probe.cxx for the request formats. Expressions that make the
native code dereference a null token (empty parentheses, invalid units passed to
`AnyFromLS`) are deliberately absent: natively they crash the process.
"""
from __future__ import annotations

from pathlib import Path

DAT = Path(__file__).resolve().parents[1] / "crates/tkernel/src/units/Units.dat"


def h(text: str | bytes) -> str:
    """Hex form of a probe string argument ("-" when empty)."""
    data = text.encode("latin-1") if isinstance(text, str) else text
    return data.hex() if data else "-"


def dictionary() -> tuple[list[bytes], list[bytes]]:
    """Quantity names and unit symbols of the vendored dictionary, in file order."""
    lines = DAT.read_bytes().splitlines()
    names: list[bytes] = []
    symbols: list[bytes] = []
    position = 0
    while position < len(lines):
        line = lines[position]
        position += 1
        if line.startswith(b"."):
            if position + 1 < len(lines):
                names.append(lines[position + 1][:40].rstrip())
            position += 3
            continue
        symbol = line[51:78].rstrip()
        if symbol and symbol not in symbols:
            symbols.append(symbol)
    return [name for name in names if name], symbols


COMPOSITES = [
    "mm", "cm", "km", "kg", "kg/cm/s", "ton/(m*s**2)", "tonne/(mm*s**2)", "sq.km", "sq.m", "cu.cm",
    "m**2", "m**3", "m/s**2", "kg*(m/s**2)", "mn", "min", "unknown_unit", "Po", "km/h", "mm\xb2",
    "1/m", "m**(-2)", "m**2**3", "(m)", "2", "0", "Pi", "m m", "-m", "+m", "-deg.C", "2m", "2mm",
    "m*m*", "m/", "**m", "(m", "m)", "((m))", "(m)(s)", "m(s)", "kg.m", "mm\xb3", "\xb5m", "dam",
    "Mm", "Gm", "3.5m", "3.5.1", "m/0", "m/(s*0)", "deg.F*s", "deg.C/s", "sq.", "cu.m", "p2",
    "mp2", "N*m", "J/(kg*deg.K)", "\xb6", "2*Pi", "rad/s", "m.s", "s-1", "", "ft", "in", "lb",
]
FORMULAS = ["1/3", "5/9", "(9/5)", "1379/3", "2*Pi", "Pi", "2**3", "-5", "+5", "(2)", "1.5e3",
            "2.5.3", "(1/3)*3", "4/0", "\xb6", "", "p2", "3.", ".5", "m", "2(3)", "(2)3"]
VALUES = ["0.0", "1.0", "-3.25", "1234.5", "1e-300", "2.5e300"]


def sentences(symbols: list[bytes]) -> list[str]:
    """Unit sentences over every dictionary symbol and the composite/invalid list."""
    lines = [f"units_sentence {h(symbol)}" for symbol in symbols]
    lines += [f"units_sentence {h(text)}" for text in COMPOSITES]
    lines += [f"units_math {h(text)}" for text in FORMULAS]
    for kind in ["formula", "units", "raw"]:
        lines += [f"units_raw {kind} {h(text)}" for text in ["m", "2*3", "kg/s", "mm\xb2", "sq.m"]]
    return lines


def conversions(symbols: list[bytes]) -> list[str]:
    """ToSI/FromSI (with the last-unit cache) and Convert over symbols and composites."""
    lines = []
    for index, symbol in enumerate(symbols):
        lines.append(f"units_tosi {VALUES[index % len(VALUES)]} {h(symbol)}")
    for text in ["mm", "mm", "km/h", "deg.C", "deg.C", "deg.F", "unknown_unit", "m m", "", "mm\xb2"]:
        lines.append(f"units_tosi 2.5 {h(text)}")
    pairs = [("m", "mm"), ("deg.C", "deg.F"), ("deg.F", "deg.C"), ("km/h", "m/s"), ("m", "s"),
             ("unknown_unit", "m"), ("m", "unknown_unit"), ("Po", "kg/cm/s"), ("deg.C", "deg.K"),
             ("sq.m", "m**2"), ("kg", "lb"), ("rad", "deg"), ("mm\xb2", "m**2"), ("", "m")]
    for value in VALUES[:4]:
        lines += [f"units_convert {value} {h(a)} {h(b)}" for a, b in pairs]
    return lines


def lookups(names: list[bytes], symbols: list[bytes]) -> list[str]:
    """Quantity, FirstQuantity and dimension look-ups."""
    lines = [f"units_quantity {h(name)}" for name in names]
    lines += [f"units_quantity {h(name)}" for name in ["BOGUS", "length", "LENGTH "]]
    lines += [f"units_first {h(symbol)}" for symbol in symbols[::3]]
    lines += [f"units_first {h(text)}" for text in ["zz", "mm", "mm", "", "m m"]]
    dims = [
        "0 1 0 0 0 0 0 0 0 0 0 1 0 0 0 0 0 0 2",
        "1 1 -2 0 0 0 0 0 0 0 1 0 0 0 0 0 0 0 0.5",
        "0 0 0 0 1 0 0 0 0 0 0 0 0 1 0 0 0 0 -1",
        "0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 3",
        "0.5 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 2",
        "1 2 -2 0 0 0 0 -1 0 1 2 -2 0 0 0 0 -1 0 1",
    ]
    lines += [f"units_dims {text}" for text in dims]
    return lines


def objects() -> list[str]:
    """Tokens, measurements and units built directly."""
    token_cases = [("m", "s", 2.0, "m", "U"), ("km", "m", 0.5, "kmx", "M"), ("deg.C", "deg.C", 3.0, "d", "U"),
                   ("mm", "0", 2.0, "m", "U"), ("kg", "kg*m", -1.5, "k", ""), ("unknown_unit", "m", 1.0, "", "U"),
                   ("m\xb2", "m", 0.25, "m\xb2x", "P"), ("deg.F", "s", 4.0, "a", "S")]
    lines = [f"units_token {h(a)} {h(b)} {v} {h(t)} {h(m)}" for a, b, v, t, m in token_cases]
    measures = [("2.5", "km", "3.0", "m", "mm", "2.0"), ("100.0", "deg.C", "32.0", "deg.F", "deg.K", "0.5"),
                ("-7.75", "kg", "2.0", "s", "g", "3.0"), ("1.0", "bogus", "2.0", "m", "mm", "2.0"),
                ("1.0", "m", "2.0", "bogus", "bogus", "2.0"), ("3.3", "m/s", "1.0", "km/h", "s", "-1.0")]
    lines += [f"units_measure {a} {h(b)} {c} {h(d)} {h(e)} {f}" for a, b, c, d, e, f in measures]
    lines += [f"units_unit {h(n)} {h(s)} {v} {m}" for n, s, v, m in
              [("metre", "m", 1.0, 0.0), ("degree", "deg.X", 1.5, 273.15), ("x", "", 0.0, -1.0)]]
    return lines


def system() -> list[str]:
    """A units-system session, including the Activate bound and shifted conversion quirks."""
    steps = [
        "new", "state", "specify LENGTH mm", "specify LENGTH m", "specify LENGTH km",
        "specify THERMODYNAMIC_TEMPERATURE deg.C", "specify MASS kg", "specify BOGUS m",
        "specify LENGTH m_m", "state", "activates", "state", "active LENGTH", "active MASS",
        "active BOGUS", "activate LENGTH m", "activate LENGTH km", "activate LENGTH zz",
        "convert LENGTH 2.5 km", "convert MASS 3.0 g", "convert TIME 4.0 s", "convert BOGUS 1.0 m",
        "convert LENGTH 1.0 zz", "remove LENGTH m", "state", "remove LENGTH zz", "remove BOGUS m",
        "remove MASS kg", "state", "specify THERMODYNAMIC_TEMPERATURE deg.F",
        "activate THERMODYNAMIC_TEMPERATURE deg.F", "convert THERMODYNAMIC_TEMPERATURE 10.0 deg.C",
        "state", "specify MASS kg", "specify LENGTH cm", "specify LENGTH dm", "activate MASS zz",
        "activate LENGTH dm", "activate LENGTH cm", "state", "named", "state", "specify AREA mm\xb2", "activates", "convert AREA 1.0 m**2",
    ]
    lines = []
    for step in steps:
        op, *args = step.split(" ")
        if op == "convert":
            quantity, value, unit = args
            encoded = f"{h(quantity.replace('_', ' '))} {value} {h(unit)}"
        else:
            encoded = " ".join(h(arg.replace("_", " ")) for arg in args)
        lines.append(f"units_system {op} {encoded}".rstrip())
    return lines


def explorers() -> list[str]:
    """Explorer walks over the dictionary and the system."""
    return [
        "units_explore dict - walk", f"units_explore dict {h('LENGTH')} walk",
        f"units_explore dict {h('BOGUS')} head", "units_explore sys - walk",
        f"units_explore sys {h('AREA')} walk", f"units_explore sys {h('BOGUS')} head",
    ]


def api() -> list[str]:
    """A UnitsAPI session: local systems, current units, conversions and dimensions."""
    lines = ["units_api ls", "units_api dimbasic", f"units_api dim {h('LENGTH')}", f"units_api dim {h('BOGUS')}"]
    units = ["mm", "m", "km/h", "deg.C", "mm\xb2", "rad", "kg/(mm*s\xb2)", "unknown_unit"]
    quantities = ["LENGTH", "SPEED", "BOGUS", "THERMODYNAMIC TEMPERATURE", "AREA"]
    for code in ["1", "2", "0", "1", "2"]:
        lines.append(f"units_api sls {code}")
        for unit in units:
            lines += [f"units_api {op} 2.5 {h(unit)}" for op in ["a2ls", "a2si", "afsi", "a2lsd", "a2sid"]]
            if unit != "unknown_unit":
                lines.append(f"units_api afls 2.5 {h(unit)}")
            lines.append(f"units_api a2a 2.5 {h(unit)} {h('m')}")
        for quantity in quantities:
            lines += [f"units_api {op} 1.5 {h(quantity)}" for op in ["c2ls", "c2si", "cfls", "cfsi", "ls2si", "si2ls"]]
            lines += [f"units_api c2any 1.5 {h(quantity)} {h('mm')}", f"units_api cfany 1.5 {h(quantity)} {h('mm')}"]
            lines += [f"units_api cu {h(quantity)}", f"units_api check {h(quantity)} {h('mm')}"]
        lines += [f"units_api scu {h('LENGTH')} {h('km')}", f"units_api scu {h('SPEED')} {h('km/h')}",
                  f"units_api scu {h('THERMODYNAMIC TEMPERATURE')} {h('deg.C')}"]
    lines += ["units_api save", "units_api reload", "units_api ls"]
    return lines


def methods() -> list[str]:
    """Every UnitsMethods table entry and the cascade unit."""
    codes = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]
    lines = [f"units_methods factor {code}" for code in codes]
    lines += [f"units_methods dump {code}" for code in codes]
    lines += [f"units_methods scale {a} {b}" for a in codes for b in [2, 6, 1]]
    factors = ["1.0", "25.4", "304.8", "1609344.0", "1000.0", "1000000.0", "0.0254", "0.001", "10.0",
               "2.54e-05", "0.0000254", "2.0", "1.0000005", "1.000002", "0.1", "0.3048"]
    lines += [f"units_methods byfactor {f} {base}" for f in factors for base in [2, 6, 1]]
    lines += [f"units_methods dumpf {f} {base}" for f in factors for base in [2, 6]]
    names = ["mm", "m", "cm", "km", "micron", "in", "min", "nin", "ft", "stat.mile", "MM", "Km", "yard", ""]
    lines += [f"units_methods fromstr {h(n)} {cs}" for n in names for cs in [0, 1]]
    lines += ["units_methods casget 2", "units_methods casset 1.0 6", "units_methods casget 2",
              "units_methods casget 1", "units_methods cascode 1", "units_methods casget 6",
              "units_methods cascode 3", "units_methods casset 2.5 2"]
    return lines


def tail() -> list[str]:
    """Dictionary dumps and lexicons.

    `DictionaryOfUnits(true)` and `Creates` on a second dictionary are absent: re-adding
    every symbol turns the unit meanings into "UU", base units stop resolving and the
    native matrix propagation then loops forever (Rust reports an error instead).
    """
    lines = ["units_lexicon formula", "units_lexicon units", "units_dict dump1", "units_dict dump2",
             "units_dict dumpdims 0 1 0 0 0 0 0 0 0", "units_dict dumpdims 9 9 9 0 0 0 0 0 0"]
    lines += [f"units_lexicon_add {h(w)} {h(m)} {v}" for w, m, v in
              [("m", "U", 0.0), ("zz", "Q", 2.0), ("\xff", "U", 1.0), ("", "U", 0.0), ("da", "M", 5.0)]]
    return lines


def cases() -> list[str]:
    """Every units request line, in the stateful order described above."""
    names, symbols = dictionary()
    head = ["units_basic", "units_tosi 2.5 -", "units_first -", "units_lexicon formula"]
    return (head + sentences(symbols) + conversions(symbols) + lookups(names, symbols) + objects()
            + system() + explorers() + api() + methods() + tail())
