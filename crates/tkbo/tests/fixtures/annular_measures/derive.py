#!/usr/bin/env python3
"""Exact monotone transformation of the retained independently bounded outer cubic side."""
from fractions import Fraction
import hashlib
import json
import math
from pathlib import Path
import struct

source=Path(__file__).parents[4]/'tkmath/tests/fixtures/prism_complete_measures/closed_cubic.json'
owned=json.loads(source.read_text())
side=[Fraction(value) for value in owned['side']['exact']]
quantities={'cap':[Fraction(9,5)]*2,'signed_sweep':[Fraction(18,5)]*2,
            'outer_side':side,'inner_side':[value/2 for value in side],
            'total':[Fraction(18,5)+Fraction(3,2)*value for value in side]}
rows=[]
for name,interval in quantities.items():
    endpoints=[]
    for index,value in enumerate(interval):
        rounded=float(value)
        if (index==0 and Fraction(rounded)>value) or (index==1 and Fraction(rounded)<value):
            rounded=math.nextafter(rounded,-math.inf if index==0 else math.inf)
        endpoints.append(struct.unpack('<Q',struct.pack('<d',rounded))[0])
    rows.append('\t'.join([name,*map(str,endpoints)]))
Path(__file__).with_name('material.tsv').write_text('\n'.join(rows)+'\n')
Path(__file__).with_name('material.json').write_text(json.dumps({
    'method':'exact affine transformation of retained certified interval',
    'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),
    'source':str(source.relative_to(Path(__file__).parents[4])),
    'exact':{key:list(map(str,values)) for key,values in quantities.items()}},indent=2)+'\n')
