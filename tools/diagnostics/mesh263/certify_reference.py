"""Certify the cylinder side coordinates with directed MPFR primitive bounds."""

from __future__ import annotations
import ctypes
import ctypes.util
import hashlib
import json
import struct
from pathlib import Path


class MpfrNumber(ctypes.Structure):
    """Mirror the public MPFR number layout at its system-library boundary."""

    _fields_ = [
        ("precision", ctypes.c_long),
        ("sign", ctypes.c_int),
        ("exponent", ctypes.c_long),
        ("limbs", ctypes.POINTER(ctypes.c_ulong)),
    ]


def reference_library() -> ctypes.CDLL:
    """Bind the installed independent library; `reference_library()` needs MPFR."""
    name = ctypes.util.find_library("mpfr")
    if name is None:
        raise ValueError("MPFR library None: expected an installed libmpfr")
    library = ctypes.CDLL(name)
    pointer = ctypes.POINTER(MpfrNumber)
    library.mpfr_init2.argtypes = [pointer, ctypes.c_long]
    library.mpfr_clear.argtypes = [pointer]
    library.mpfr_set_d.argtypes = [pointer, ctypes.c_double, ctypes.c_int]
    library.mpfr_get_d.argtypes = [pointer, ctypes.c_int]
    library.mpfr_get_d.restype = ctypes.c_double
    library.mpfr_get_version.restype = ctypes.c_char_p
    for function in ("sin", "cos"):
        getattr(library, "mpfr_" + function).argtypes = [pointer, pointer, ctypes.c_int]
    return library


def binary_word(value: float) -> str:
    """Encode a stored binary64 value; `binary_word(0.0)` is sixteen zeroes."""
    return struct.pack(">d", value).hex()


def binary_real(word: str) -> float:
    """Decode one exact input; `binary_real('3ff0000000000000')` is one."""
    return struct.unpack(">d", bytes.fromhex(word))[0]


def directed_words(
    library: ctypes.CDLL, function: str, argument: float, precision: int
) -> list[str]:
    """Enclose then round both ends independently; `directed_words(mpfr, 'sin', .5, 256)`."""
    source, result = MpfrNumber(), MpfrNumber()
    library.mpfr_init2(ctypes.byref(source), precision)
    library.mpfr_init2(ctypes.byref(result), precision)
    try:
        library.mpfr_set_d(ctypes.byref(source), argument, 0)
        words: list[str] = []
        for rounding in (3, 2):
            getattr(library, "mpfr_" + function)(
                ctypes.byref(result), ctypes.byref(source), rounding
            )
            words.append(binary_word(library.mpfr_get_d(ctypes.byref(result), 0)))
        return words
    finally:
        library.mpfr_clear(ctypes.byref(source))
        library.mpfr_clear(ctypes.byref(result))


def certify_primitive(
    library: ctypes.CDLL, function: str, argument: float
) -> dict[str, object]:
    """Require directed 256/512-bit bounds to round identically; `certify_primitive(mpfr, 'sin', .5)`."""
    bounds = {
        str(precision): directed_words(library, function, argument, precision)
        for precision in (256, 512)
    }
    words = {word for pair in bounds.values() for word in pair}
    if len(words) != 1:
        raise ValueError(
            f"MPFR bounds {bounds!r}: expected one certified binary64 cell"
        )
    return {
        "parameter": binary_word(argument),
        "function": function,
        "primitive": words.pop(),
        "directed_rounded_bounds": bounds,
    }


def side_node_certificate(
    library: ctypes.CDLL, index: int, row: str
) -> list[dict[str, object]]:
    """Retain the periodic seam's copied endpoint; `side_node_certificate(mpfr, 1, row)`."""
    x, y, z, u, v = map(binary_real, row.split())
    argument = binary_real("401921fb54442d18") if u == 0.0 else u
    certificates: list[dict[str, object]] = []
    for function, expected in (("cos", x), ("sin", y)):
        certificate = certify_primitive(library, function, argument)
        scaled = binary_word(5.0 * binary_real(str(certificate["primitive"])))
        if scaled != binary_word(expected):
            raise ValueError(
                f"node {index} {function}: {scaled}, expected {binary_word(expected)}"
            )
        certificates.append(
            dict(certificate, node=index, scaled=scaled, native=binary_word(expected))
        )
    if z != v or z not in (0.0, 10.0):
        raise ValueError(
            f"node {index} z/v {(z, v)!r}: expected matching cylinder levels"
        )
    return certificates


def certify_reference(reference: Path, output: Path) -> None:
    """Check all 54 side nodes without Rust output; `certify_reference(reference, certificate)`."""
    lines = reference.read_text().splitlines()
    if lines[0] != "face 0 54 52":
        raise ValueError(
            f"first face {lines[0]!r}: expected cylinder side 54 nodes/52 triangles"
        )
    library = reference_library()
    records = [
        record
        for index, row in enumerate(lines[1:55], 1)
        for record in side_node_certificate(library, index, row)
    ]
    write_certificate(reference, output, library, records)


def write_certificate(
    reference: Path,
    output: Path,
    library: ctypes.CDLL,
    records: list[dict[str, object]],
) -> None:
    """Keep library identity with each directed calculation; `write_certificate(ref, out, mpfr, records)`."""
    result = {
        "oracle": "MPFR directed 256/512-bit sin/cos bounds",
        "version": library.mpfr_get_version().decode(),
        "reference_sha256": hashlib.sha256(reference.read_bytes()).hexdigest(),
        "composition": "rounded primitive, then binary64 radius-five product; periodic seam copies endpoint",
        "checked_coordinates": len(records),
        "records": records,
    }
    output.write_text(json.dumps(result, indent=2) + "\n")


if __name__ == "__main__":
    root = Path(__file__).resolve().parent
    certify_reference(
        root / "portable-cylinder.expected", root / "side-primitive-certificates.json"
    )
