"""Deterministic request corpus for the TKMeshVS numeric oracle.

Each request is one `operation numbers...` line read by tools/oracle/tkmeshvs_probe.cxx and
crates/tkmeshvs/examples/numeric_probe.rs. The verifier deals the requests round-robin to
parallel shards (tools/probe_shards.py), so a request prints exactly one line and never relies
on an earlier request. The probe mesh (tools/oracle/tkmeshvs_probe_source.cxx) has nodes 1..14
and elements 101..108; masks select them bit by bit.
"""
from __future__ import annotations
from itertools import product

ALL_NODES = 0x3FFF
ALL_ELEMENTS = 0xFF
# MeshVS_DMF_*: wireframe, shading, shrink, vectors, nodal and elemental colors, texts,
# highlight and selection flags, deformed modes.
BUILD_MODES = (0, 1, 2, 3, 4, 8, 16, 32, 1 | 0x400, 2 | 0x400, 3 | 0x400, 1 | 0x200, 3 | 0x200,
               0x80, 0x100, 0x180, 0x8 | 0x400)
DRAWER_BITS = (0, 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192,
               1 | 2 | 16, 4 | 8, 2 | 16 | 32, 128 | 4096, 512 | 1024)


def append_source_requests(requests: list[str]) -> None:
    """The default normal, bound and group members, the topologies and the deformed source.

    A negative MaxNodes is not requested: the native GetNormalsByElement allocates its
    MeshVS_Buffer of the negative size before its own `MaxNodes <= 0` test and raises
    Standard_OutOfMemory from the allocator; the port returns false without allocating."""
    for variant, nodes, elements, max_nodes in product((0, 1, 2, 3), (ALL_NODES, 0x3FF0, 0), (ALL_ELEMENTS, 0x0F), (10, 6, 4, 3, 0)):
        requests.append(f"mv_ds {variant} {nodes} {elements} {max_nodes}")
    for kind, n in product((0, 1), (-1, 0, 2, 3, 4, 5, 8)):
        requests.append(f"mv_topology {kind} {n}")
    for magnify, mask, magnify2 in product((2, -1, 0.5), (ALL_NODES, 0x5, 0, 0x3FFE), (0, 3, -2)):
        requests.append(f"mv_deformed {magnify} {mask} {magnify2}")


def append_build_requests(requests: list[str]) -> None:
    """Every builder kind and variant over modes, ids, drawer changes, hidden and excluded ids."""
    variants = {0: (0,), 1: (0, 1, 2, 3), 2: (0, 1, 2), 3: (0, 1), 4: (0, 1, 2, 3), 5: (0,)}
    for kind, mode, is_element in product(range(6), BUILD_MODES, (0, 1)):
        ids = ALL_ELEMENTS if is_element else ALL_NODES
        for variant in variants[kind]:
            requests.append(f"mv_build {kind} {variant} {mode} {is_element} {ids} 0 -1 -1 0 -1")
    for kind, bits, is_element in product(range(6), DRAWER_BITS, (0, 1)):
        ids = ALL_ELEMENTS if is_element else ALL_NODES
        mode = {0: 2, 1: 8, 2: 16, 3: 32, 4: 4, 5: 0x100}[kind]
        for display in sorted({mode, 1, 3, mode | 0x400}):
            requests.append(f"mv_build {kind} {variants[kind][-1]} {display} {is_element} {ids} {bits} -1 -1 0 -1")
    hidden = ((-1, -1), (0, -1), (0x7, 0x9), (ALL_NODES, ALL_ELEMENTS), (0x40, 0x84))
    for kind, (nodes, elements), is_element, excl, excluding in product(range(5), hidden, (0, 1), (0, 0x5), (-1, 0, 1)):
        ids = ALL_ELEMENTS if is_element else ALL_NODES
        mode = {0: 3, 1: 8, 2: 16, 3: 32, 4: 4}[kind]
        requests.append(f"mv_build {kind} 0 {mode} {is_element} {ids} 0 {nodes} {elements} {excl} {excluding}")
    for kind, single in product(range(6), (0x1, 0x2, 0x4, 0x8, 0x20, 0x40, 0x80)):
        for mode in (1 | 0x400, 2 | 0x400, 3 | 0x400, 2 | 0x200):
            requests.append(f"mv_build {kind} 0 {mode} 1 {single} 0 -1 -1 0 -1")
            requests.append(f"mv_build {kind} 0 {mode} 0 {single} 0 -1 -1 0 -1")


def append_mesh_requests(requests: list[str]) -> None:
    """MeshVS_Mesh::Compute over builder sets, and the builder and visibility members."""
    for mode, builders, bits, overlapped in product((0, 1, 2, 3, 4, 8, 16, 32, 2 | 0x400, 1024), (0, 1, 3, 31, 16, 8 | 1), (0, 1, 4, 8), (0, 1)):
        requests.append(f"mv_compute {mode} {builders} {bits} {overlapped}")
    sequences = ((0, 1, 2, 3), (0, 3, 4, 5, 6, 7, 8, 9, 17), (2, 0, 4, 4, 4), (10, 11, 12, 13, 14, 15, 18, 12),
                 (14, 10, 11, 18, 16, 19, 12), (3, 3, 5, 5, 7, 6), (0, 1, 2, 3, 0, 1, 2, 3, 5, 4, 9))
    for ops in sequences:
        requests.append(f"mv_mesh_api {len(ops)} " + " ".join(map(str, ops)))


def append_selection_requests(requests: list[str]) -> None:
    """ComputeSelection by mode and method, owner highlight and the sensitive entities."""
    modes = (0, 1, 2, 4, 8, 16, 0x100, 30, 3)
    for mode, method, variant in product(modes, (0, 1, 2, 3), (0, 1, 3, 4, 7)):
        requests.append(f"mv_select {mode} {method} {variant} 10 0 -1 1 -1")
    for mode, max_nodes, hidden, hilighter, selectable in product(modes, (10, 6, 4, 0), ((0, -1), (-1, -1), (0x7, 0x9), (ALL_NODES, 0x80)), (0, 1), (-1, 0x3F, 0)):
        requests.append(f"mv_select {mode} 0 1 {max_nodes} {hidden[0]} {hidden[1]} {hilighter} {selectable}")
    owners = ((1, 0, 2), (1, 0, 14), (8, 0, 102), (8, 0, 101), (8, 0, 103), (8, 0, 108), (4, 0, 104), (16, 0, 106),
              (2, 0, 107), (0x100, 1, 202), (0x100, 1, 201), (0x100, 1, 204), (0x100, 1, 205), (0, 0, 0), (8, 4, 0), (1, 7, 0))
    for (mode, variant, key), immediate in product(owners, (0, 1)):
        requests.append(f"mv_highlight {mode} {variant} {key} {immediate}")
    for kind, selectable in product(range(11), (-1, ALL_NODES, 0x7, 0)):
        requests.append(f"mv_sensitive {kind} {selectable}")


def append_tkmeshvs_requests(requests: list[str]) -> None:
    """Append every TKMeshVS request family in a fixed order."""
    append_source_requests(requests)
    append_build_requests(requests)
    append_mesh_requests(requests)
    append_selection_requests(requests)
