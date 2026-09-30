"""Deterministic requests for ShapeExtend (tools/oracle/tkshhealing_probe_shapeextend.cxx).

`sh_status` encodes every ShapeExtend_Status enumerator and decodes it from bit fields:
no bit, each single bit, each byte, every bit, bits beyond the 16 the flags use, and a
negative field.

`sh_msg` runs ShapeExtend::Init, which loads the compiled SHMessage resource, then asks the
catalog for every key the resource defines (read from the embedded copy
crates/tkshhealing/src/sh_message/SHAPE.us) and for keys it does not define. Msg binds a
missing key on the fly, so each missing key is asked once (a later shard request would
otherwise see it bound).
"""
from __future__ import annotations
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SHAPE_US = ROOT/'crates/tkshhealing/src/sh_message/SHAPE.us'
# The 19 enumerators of ShapeExtend_Status, OK..FAIL.
STATUS_COUNT = 19
# Keys the resource does not define: near misses of defined keys, the empty key and
# a key with a space.
MISSING_KEYS = ['ShapeFix.FixSmallSolid.MSG9', 'shapefix.fixsmallsolid.msg0', 'Name.Shape',
                'ShapeFix.FixSmallSolid.MSG0 ', '', 'Message_Msg_BadKeyword.Probe']


def status_request(flag: int, status: int) -> str:
    """One `sh_status flag status` request line."""
    return f'sh_status {flag} {status}'


def status_flags() -> list[int]:
    """The bit fields decoded against every status."""
    single = [1 << bit for bit in range(16)]
    return [0, 0x00ff, 0xff00, 0xffff, 0x10000, 0x10001, -1, 0x0101, 0x8001] + single


def message_request(key: str) -> str:
    """One `sh_msg n (byte)...` request line spelling `key`."""
    data = key.encode('latin1')
    return ' '.join(['sh_msg', str(len(data)), *map(str, data)])


def resource_keys() -> list[str]:
    """The keys of the compiled resource, in file order (a line `.key` opens a message)."""
    lines = SHAPE_US.read_text(encoding='latin1').splitlines()
    return [line[1:].strip() for line in lines if line.startswith('.')]


def append_shapeextend_requests(requests: list[str]) -> None:
    """Append the ShapeExtend requests: every flag against every status, then the messages."""
    requests.extend(status_request(flag, status)
                    for flag in status_flags() for status in range(STATUS_COUNT))
    keys = list(dict.fromkeys(resource_keys() + MISSING_KEYS))
    requests.extend(message_request(key) for key in keys)
