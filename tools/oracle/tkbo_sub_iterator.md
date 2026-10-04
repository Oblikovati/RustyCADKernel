# BOPDS_SubIterator native subset replay

Oracle: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The source probe uses two overlapping unit boxes and five subset choices:
all face/edge/vertex indices, vertices, duplicated edges, faces, and empty.
Each of the 25 subset combinations runs Prepare/Initialize and a second explicit
Intersect/Initialize. All 50 output lines compare expected length and the entire
ordered pair sequence. Reverse insertion, duplicate indices, identical subset
members, subshape filtering, and append behavior are exercised.

Rust borrows DS and subset lists through lifetimes and accepts `None` for SetDS.
Undefined native access to absent inputs or exhausted cursors is represented by
Rust failures. Concurrent source mutation, invalid iterator lifetime after pool
mutation, allocator fault injection, derived virtual overrides, and full legacy
coverage remain unproven. The private generated EE-vertex traversal now uses
this shared implementation; its 161-row PerformVF replay remains green.

Only face/edge/vertex records have usable boxes in this fixture. An initial
all-record probe threw native `Standard_ConstructionError: Bnd_Box is void` for
an unprepared aggregate bound; this failure was not converted into an empty
success or included as a passing geometric scenario.

Build this C++ source against the pinned modeling installation with TKBO, TKPrim,
TKTopAlgo, TKBRep, TKMath and TKernel; compare stdout with its expected fixture.
Rust command: `cargo test -p rustycad-tkbo --test bop_ds_sub_iterator`.
These are supplemental fixtures, not additional mapped legacy declarations.
