"""Sensitive-set map order, counters, native RTTI and retained BVH handles."""
from __future__ import annotations
from itertools import product
from random import Random


def entity_trace(count: int, builder: int, rejected: int, flags: int, operations: list[tuple[int,int,int]]) -> str:
    """Encode fixed-width operations so failure leaves the rest of the trace readable."""
    suffix = ' '.join(str(value) for operation in operations for value in operation)
    return f'v3_sensitive_entity_set {count} {builder} {rejected} {flags} {len(operations)} {suffix}'


def append_sensitive_entity_set_requests(requests: list[str]) -> None:
    """Exercise empty/duplicate/reordered sets, all query failures, builder changes and flags."""
    for count, builder, rejected, flags in product((1,3,7,12), range(3), (0,1), range(4)):
        operations = [(12,0,0),(1,0,count),(6,0,0),(1,0,count),(3,0,0),(11,0,0),
                      (13,0,1),(2,0,1),(6,0,0),(7,2,0),(5,0,0),(6,0,0),
                      (8,0,0),(8,0,0),(9,0,0),(9,0,0),(9,2,0),(14,1,0),(14,0,0),
                      (4,-1,0),(4,0,count),(10,0,-1),(10,0,0),(10,0,1),(10,0,5),
                      (11,count,0),(2,0,count),(6,0,0),(12,0,0)]
        requests.append(entity_trace(count,builder,rejected,flags,operations))
    rng=Random(611)
    for _ in range(512):
        count=rng.choice((1,3,7,12)); operations=[]
        for _ in range(48):
            op=rng.randrange(15);a=rng.randrange(count);b=rng.randrange(count+2)
            if op in (4,10,11):a=rng.randrange(-1,count+1)
            if op in (7,8,9):a=rng.randrange(3)
            if op==3:b=rng.randrange(4)
            if op==13:b=rng.randrange(3)
            operations.append((op,a,b))
        requests.append(entity_trace(count,rng.randrange(3),rng.randrange(2),rng.randrange(4),operations))
