"""Check independent Arb enclosures against exact rational errors and directed binary64 bounds."""
from fractions import Fraction
import json
from pathlib import Path
import struct


def dyadic(endpoint: dict[str, object]) -> Fraction:
    """Interpret returned exact dyadic endpoints without floating-point evaluation."""
    return Fraction(int(endpoint["mantissa"])) * Fraction(2) ** int(endpoint["exponent"])


def binary64(bits: int) -> Fraction:
    """Read the exact dyadic value of a finite binary64 bit pattern."""
    return Fraction.from_float(struct.unpack(">d", struct.pack(">Q", bits))[0])


def verify() -> None:
    """Prove each chosen upper error bound covers the independent enclosure; its predecessor does not."""
    evidence = json.loads(Path(__file__).with_name("independent-error-enclosures.json").read_text())
    cases = [(Fraction(1, 5 * 2**50), 0x3CA999999999999A),
             (Fraction(1, 5 * 2**48), 0x3CC999999999999A)]
    for entry, (exact, bits) in zip(evidence["entries"], cases, strict=True):
        result = entry["output"]
        assert result["status"] == "ENCLOSED", result
        lower, upper = dyadic(result["lower"]), dyadic(result["upper"])
        assert binary64(bits - 1) < lower <= exact <= upper <= binary64(bits)


if __name__ == "__main__":
    verify()
    print("Both independent error enclosures and directed binary64 bounds verified")
