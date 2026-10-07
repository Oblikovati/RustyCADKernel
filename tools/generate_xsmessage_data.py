#!/usr/bin/env python3
"""Carry the compiled data exchange message resources over from the pinned OCCT sources.

Usage: python3 tools/generate_xsmessage_data.py [--source ../OCCT] [--check]

The XSMessage package of TKXSBase holds no code: XSMessage_XSTEP_us.pxx and
XSMessage_XSTEP_fr.pxx each define one C string literal, generated from
resources/XSMessage/XSTEP.us and XSTEP.fr. Interface_StaticStandards loads the `us` literal
into the message catalog when the CSF_XSMessage directory does not give the file. As for
SHMessage (tools/generate_shmessage_data.py, whose literal decoder this tool reuses), the
bytes are decoded from the literals and stored as crates/tkxsbase/src/xs_message/XSTEP.us
and XSTEP.fr, which the crate embeds with `include_bytes!`.
porting/xsmessage-data.json records the source hashes and the payload hashes; `--check`
regenerates everything in memory and fails on any difference.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

from generate_shmessage_data import literal_bytes

ROOT = Path(__file__).resolve().parent.parent
TARGET = ROOT/'crates/tkxsbase/src/xs_message'
PROVENANCE = ROOT/'porting/xsmessage-data.json'
PACKAGE = 'src/DataExchange/TKXSBase/XSMessage'
# (literal name, header, embedded file)
RESOURCES = [('XSMessage_XSTEP_us', 'XSMessage_XSTEP_us.pxx', 'XSTEP.us'),
             ('XSMessage_XSTEP_fr', 'XSMessage_XSTEP_fr.pxx', 'XSTEP.fr')]


def generate(source: Path) -> dict[Path, bytes]:
    """Return the embedded files and their provenance file; generate(Path('../OCCT'))."""
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
    return files


def main() -> None:
    """Write or verify the byte-exact embedded resources and their provenance."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=ROOT/'../OCCT')
    parser.add_argument('--check', action='store_true')
    options = parser.parse_args()
    for path, expected in generate(options.source.resolve()).items():
        if not options.check:
            path.write_bytes(expected)
        elif not path.is_file() or path.read_bytes() != expected:
            raise RuntimeError(f'Stale generated message resource: {path}; rerun without --check')


if __name__ == '__main__':
    main()
