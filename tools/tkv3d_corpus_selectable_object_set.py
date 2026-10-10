"""Five-subset transitions, live iterators and camera/group-dependent BVH updates."""
from itertools import product
from random import Random


def object_set_trace(count: int, seed: int, projection: int, operations: list[tuple[int,int,int]]) -> str:
    suffix=' '.join(str(value) for operation in operations for value in operation)
    return f'v3_selectable_object_set {count} {seed} {projection} {len(operations)} {suffix}'


def append_selectable_object_set_requests(requests: list[str]) -> None:
    for count,seed,projection in product((1,5,12),range(5),range(2)):
        operations=[(12,0,-1),(4,0,0),(5,0,0),(5,1,0)]
        operations += [(0,i,0) for i in range(count)]
        operations += [(5,1,0),(15,0,0),(16,0,0),(18,0,0),(17,0,0),
            (6,4,3),(5,1,0),(7,24,0),(5,1,0),(5,0,1),(5,1,1),
            (10,0,3),(11,0,19),(20,0,-17),(3,0,0),(4,0,0),(5,1,2),(12,0,2),
            (2,0,4),(3,0,0),(5,1,2),(19,0,0),(5,1,2),
            (2,0,10),(3,0,0),(4,0,0),(5,1,3),(12,0,1),(2,0,1),(5,1,3),
            (21,0,0),(13,0,-1),(13,4,count),(14,0,0),(5,1,2),(12,0,0)]
        requests.append(object_set_trace(count,seed,projection,operations))
    rng=Random(821)
    for _ in range(512):
        count=rng.choice((1,5,12)); operations=[(0,i,0) for i in range(count)]
        for _ in range(48):
            op=rng.randrange(22);a=rng.randrange(count);b=rng.randrange(10)
            if op==5:a=rng.randrange(2);b=rng.randrange(-2,4)
            if op==13:a=rng.randrange(5);b=rng.randrange(-1,count+1)
            if op==9:b=rng.choice((-10,0,3,12,99))
            if op==12:b=rng.choice((-1,0,1,3))
            if op==10:b=rng.randrange(4)
            if op==2:b=rng.randrange(11)
            if op in (11,20):b=rng.randrange(-24,25)
            operations.append((op,a,b))
        requests.append(object_set_trace(count,rng.randrange(5),rng.randrange(2),operations))
