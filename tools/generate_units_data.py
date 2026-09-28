#!/usr/bin/env python3
"""Decode OCCT's compiled unit resource, retaining its exact Latin-1 byte layout.

The editable resources/UnitsAPI/Units.dat contains literal C escapes; parsing that
file directly shifts fixed-width fields. The compiled .pxx is the behavioral input.
Usage: python3 tools/generate_units_data.py [--check]
"""
import argparse
import ast
import hashlib
import json
from pathlib import Path
import re
from porting import ROOT

def generate(source: Path) -> tuple[bytes,dict]:
    """Return compiled string bytes and provenance; generate(source)."""
    resource = source/'src/FoundationClasses/TKernel/UnitsAPI/UnitsAPI_Units_dat.pxx'
    text = resource.read_text()
    literals = re.findall(r'^  (".*")\s*;?$',text,re.M)
    payload = b''.join(ast.literal_eval(literal).encode('latin1') for literal in literals)
    provenance = {'source':str(resource.relative_to(source)),
                  'source_sha256':hashlib.sha256(resource.read_bytes()).hexdigest(),
                  'payload_sha256':hashlib.sha256(payload).hexdigest(),
                  'lines':len(payload.splitlines())}
    return payload,provenance

def main() -> None:
    """Write or verify the byte-exact embedded resource and provenance."""
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source',type=Path,default=ROOT/'../OCCT')
    parser.add_argument('--check',action='store_true')
    options=parser.parse_args()
    payload,provenance=generate(options.source.resolve())
    artifacts={ROOT/'crates/tkernel/src/units/Units.dat':payload,
               ROOT/'porting/units-data.json':(json.dumps(provenance,indent=2)+'\n').encode()}
    for path,expected in artifacts.items():
        if options.check:
            if path.read_bytes()!=expected: raise RuntimeError(f'Stale generated unit resource: {path}')
        else: path.write_bytes(expected)
if __name__=='__main__': main()
