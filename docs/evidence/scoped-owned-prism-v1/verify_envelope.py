"""Offline framing/cross-binding check, not a geometry validator or Rust evaluator."""
import hashlib
from fractions import Fraction
import json
from pathlib import Path
import re
import struct
import sys


def block(payload: bytes, offset: int) -> tuple[bytes, int]:
    size = struct.unpack_from("<Q", payload, offset)[0]
    end = offset + 8 + size
    assert end <= len(payload), (offset, size, len(payload))
    return payload[offset + 8:end], end


def children(payload: bytes) -> dict[str, bytes]:
    header = b"Martensite.ScopedOwnedPrism/v1\0"
    assert payload.startswith(header)
    offset, rows = len(header), {}
    for name in ("request", "publication", "names", "reservations", "body", "volume"):
        rows[name], offset = block(payload, offset)
    assert offset == len(payload), (offset, len(payload))
    return rows


def first_block(payload: bytes, header: bytes) -> bytes:
    prefix = header + struct.pack("<I", 1)
    assert payload.startswith(prefix), header
    child, _ = block(payload, len(prefix))
    return child


def named_publication(names: bytes) -> bytes:
    envelope = first_block(names, b"Martensite.SemanticBodyMap\0")
    prefix = b"Martensite.SemanticNames\0" + struct.pack("<I", 2)
    assert envelope.startswith(prefix)
    offset = len(prefix)
    assert struct.unpack_from("<5Q", envelope, offset) == (257, 0, 7000, 901, 19)
    offset += 40
    assert envelope[offset:offset + 17] == b"\1" + struct.pack("<2d", 1., 1.)
    offset += 17
    count = struct.unpack_from("<Q", envelope, offset)[0]
    assert count == 101
    child, end = block(envelope, offset + 8 + count * 35)
    assert end == len(envelope)
    return child


def verify_bindings(rows: dict[str, bytes]) -> None:
    assert named_publication(rows["names"]) == rows["publication"]
    assert first_block(rows["body"], b"Martensite.NamedPrismBody\0") == rows["names"]
    prefix = b"Martensite.SemanticReservationCoverage\0"
    assert first_block(rows["reservations"], prefix) == rows["names"]
    assert len(rows["request"]) == 615
    publication = b"Martensite complete named prism publication V1\n"
    assert rows["publication"].startswith(publication + struct.pack("<I", 2))


def request_oracle(payload: bytes) -> None:
    prefix = b"Martensite.OwnedPolynomialPrismRequest/v1\0"
    assert payload.startswith(prefix)
    offset = len(prefix)
    assert struct.unpack_from("<I7Q", payload, offset) == (1, 257, 1, 0, 71, 7000, 901, 19)
    offset += 60
    assert struct.unpack_from("<BQ", payload, offset) == (0, 4)
    offset += 9
    assert struct.unpack_from("<12d", payload, offset) == (0., 0., 0., 4., 0., 0., 0., 4., 0., 0., 0., 0.)
    offset += 96
    assert struct.unpack_from("<12d", payload, offset) == (0., 0., 0., 0., 0., 1., 1., 0., 0., 0., 1., 0.)
    offset += 96
    assert struct.unpack_from("<B5d", payload, offset) == (1, 1., 1., 0., 0., 2.)


def trace_counts(path: Path) -> dict[str, int]:
    pattern = r"CompletePrismCheckpoint \{ phase: (.*?), ordinal: (\d+), child_ordinal: (.*?) \}"
    points = re.findall(pattern, path.read_text())
    counts: dict[str, int] = {}
    for ordinal, (phase, observed, _) in enumerate(points):
        assert ordinal == int(observed)
        category = phase.split("(", 1)[0]
        counts[category] = counts.get(category, 0) + 1
    expected = dict(Preparation=3696, Resolution=1700, Publication=2866,
                    Names=1, Reservations=1, Body=1, Volume=2, Commit=1)
    assert counts == expected, counts
    return counts


def publication_children(payload: bytes) -> dict[str, bytes]:
    offset = len(b"Martensite complete named prism publication V1\n") + 4
    rows = {}
    for name in ("join", "validation", "history", "measures"):
        rows[name], offset = block(payload, offset)
    assert offset == len(payload)
    return rows


def volume_operands(payload: bytes) -> tuple[tuple[float, ...], dict[str, bytes]]:
    prefix = b"Martensite joined conditional volume V1\0"
    assert payload.startswith(prefix)
    offset = len(prefix) + 4 + 8  # Four sign tags and the signed winding integer.
    scalar = struct.unpack_from("<4d", payload, offset)
    offset, rows = offset + 32, {}
    for name in ("join", "orientation", "measures"):
        rows[name], offset = block(payload, offset)
    assert offset == len(payload)
    return scalar, rows


def exact_measure_oracle(rows: dict[str, bytes]) -> dict[str, object]:
    # x=12t(1-t)^2, y=12t^2(1-t): Green integrand/2=72t^2(1-t)^2.
    area = 72 * (Fraction(1, 3) - Fraction(1, 2) + Fraction(1, 5))
    assert area == Fraction(12, 5)
    scalar, volume = volume_operands(rows["volume"])
    publication = publication_children(rows["publication"])
    assert volume["join"] == publication["join"]
    assert volume["measures"] == publication["measures"]
    lower, upper, representative, error = map(Fraction.from_float, scalar)
    assert lower <= 2 * area <= upper
    assert abs(representative - 2 * area) <= error
    caps = cap_oracle(volume["measures"], area, publication["join"])
    return dict(area=str(area), volume=str(2 * area), caps=caps,
                volume_error=str(error), volume_bound=[str(lower), str(upper)])


def cap_oracle(payload: bytes, area: Fraction, join: bytes) -> list[dict[str, object]]:
    prefix = b"martensite.joined-prism-measures/v1"
    assert payload.startswith(prefix)
    bound, offset = block(payload, len(prefix))
    assert bound == join
    assert struct.unpack_from("<I", payload, offset)[0] == 1
    offset, caps = offset + 4, []
    for _ in range(2):
        lo, hi, error = map(Fraction.from_float, struct.unpack_from("<3d", payload, offset))
        assert lo <= area <= hi and abs(lo - area) <= error
        options = struct.unpack_from("<IHHd", payload, offset + 24)
        assert options[:3] == (1, 16, 1)
        assert error <= Fraction.from_float(options[3])
        assert payload[offset + 24:offset + 40] == payload[offset + 40:offset + 56]
        caps.append(dict(bounds=[str(lo), str(hi)], error=str(error)))
        offset += 56
    return caps


def main() -> None:
    payload = Path(sys.argv[1]).read_bytes()
    rows = children(payload)
    verify_bindings(rows)
    request_oracle(rows["request"])
    output = {name: dict(bytes=len(value), sha256=hashlib.sha256(value).hexdigest())
              for name, value in rows.items()}
    output["checkpoint_counts"] = trace_counts(Path(sys.argv[2]))
    output["exact_closed_form"] = exact_measure_oracle(rows)
    Path(sys.argv[3]).write_text(json.dumps(output, indent=2) + "\n")
    Path(sys.argv[4]).write_text("actual-semantic-resolver-v2=" + rows["reservations"].hex() + "\n")


if __name__ == "__main__":
    main()
