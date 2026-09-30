#!/usr/bin/env python3
"""Carry the compiled Shape Healing message resources over from the pinned OCCT sources.

Usage: python3 tools/generate_shmessage_data.py [--source ../OCCT] [--check]

The SHMessage package of TKShHealing holds no code: SHMessage_SHAPE_us.pxx and
SHMessage_SHAPE_fr.pxx each define one C string literal, generated from
resources/SHMessage/SHAPE.us and SHAPE.fr. ShapeExtend::Init loads the `us` literal into
the message catalog when the CSF_SHMessage directory does not give the file. The compiled
literal, not the editable resource, is the behavioural input, so the bytes are decoded
from the literals (C escapes included) and stored as crates/tkshhealing/src/sh_message/
SHAPE.us and SHAPE.fr, which the crate embeds with `include_bytes!`.
porting/shmessage-data.json records the source hashes and the payload hashes; `--check`
regenerates everything in memory and fails on any difference.
"""
from __future__ import annotations

import argparse
import ast
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent.parent
TARGET = ROOT/'crates/tkshhealing/src/sh_message'
PROVENANCE = ROOT/'porting/shmessage-data.json'
PACKAGE = 'src/ModelingAlgorithms/TKShHealing/SHMessage'
# (literal name, header, embedded file)
RESOURCES = [('SHMessage_SHAPE_us', 'SHMessage_SHAPE_us.pxx', 'SHAPE.us'),
             ('SHMessage_SHAPE_fr', 'SHMessage_SHAPE_fr.pxx', 'SHAPE.fr')]
LITERAL = re.compile(r'^\s*(".*")\s*;?\s*$', re.M)


def literal_bytes(text: str, name: str) -> bytes:
    """The bytes of the string literal `name` of a .pxx; literal_bytes(pxx_text, 'X') == b'...'."""
    start = text.find(f'static const char {name}[] =')
    if start < 0:
        raise ValueError(f'No literal {name!r} in the resource header; expected `static const char {name}[] =`')
    end = text.index(';', start)
    pieces = LITERAL.findall(text[start:end + 1])
    # Adjacent C literals concatenate; every character of the pinned files is ASCII, so
    # Latin-1 keeps each byte (an escape such as \351 would too).
    return b''.join(ast.literal_eval(piece).encode('latin1') for piece in pieces)


def generate(source: Path) -> tuple[dict[Path, bytes], dict]:
    """Return the embedded files and their provenance; generate(Path('../OCCT'))."""
    files: dict[Path, bytes] = {}
    provenance: dict[str, dict] = {}
    for name, header, output in RESOURCES:
        path = source/PACKAGE/header
        payload = literal_bytes(path.read_text(encoding='latin1'), name)
        files[TARGET/output] = payload
        provenance[output] = {'source': f'{PACKAGE}/{header}',
                              'source_sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
                              'payload_sha256': hashlib.sha256(payload).hexdigest(),
                              'bytes': len(payload), 'lines': len(payload.splitlines())}
    files[PROVENANCE] = (json.dumps(provenance, indent=2)+'\n').encode()
    return files, provenance


def main() -> None:
    """Write or verify the byte-exact embedded resources and their provenance."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=ROOT/'../OCCT')
    parser.add_argument('--check', action='store_true')
    options = parser.parse_args()
    files, _ = generate(options.source.resolve())
    for path, expected in files.items():
        if not options.check:
            path.write_bytes(expected)
        elif not path.is_file() or path.read_bytes() != expected:
            raise RuntimeError(f'Stale generated message resource: {path}; rerun without --check')


if __name__ == '__main__':
    main()
