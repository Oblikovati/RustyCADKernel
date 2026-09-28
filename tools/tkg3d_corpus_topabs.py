"""Deterministic requests for the TopAbs package functions (tools/oracle/tkg3d_probe_topabs.cxx).

g3_topabs covers the whole orientation algebra and every name; g3_topabs_parse feeds
strings, as byte lists, to both FromString overloads: every name in several cases,
names with a NUL inside (the C string ends there), near misses, padding, the empty
string and non-ASCII UTF-8 (only ASCII letters change case natively).
"""
from __future__ import annotations
from tkg2d_corpus import fmt

NAMES = ['COMPOUND', 'COMPSOLID', 'SOLID', 'SHELL', 'FACE', 'WIRE', 'EDGE', 'VERTEX', 'SHAPE',
         'FORWARD', 'REVERSED', 'INTERNAL', 'EXTERNAL', 'ON', 'IN', 'OUT', 'UNKNOWN']


def spellings():
    """Every name upper, lower and mixed case, then the edge cases."""
    texts = []
    for name in NAMES:
        mixed = ''.join(c.lower() if i % 2 else c for i, c in enumerate(name))
        texts += [name, name.lower(), mixed]
    texts += ['', ' FACE', 'FACE ', 'FACES', 'FAC', 'Edge\0junk', 'forward\0', '\0FACE', 'SHAPEE',
              'compsolid ', 'Réversed', 'ßhell', 'FAÇE', 'external\t', 'VERTEX\nVERTEX', '0', '-1']
    return texts


def append_topabs_requests(requests: list[str]) -> None:
    """Append the TopAbs requests in a fixed order."""
    requests.append('g3_topabs')
    for text in spellings():
        data = text.encode('utf-8')
        requests.append('g3_topabs_parse ' + fmt([len(data), *data]))
