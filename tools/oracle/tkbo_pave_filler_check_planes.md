# Plane-pair preparation oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`,
`BOPAlgo_PaveFiller_6.cxx::CheckPlanes`. The derived native probe exposes the
protected method and records lazy FaceInfo initialization, same-face queries, and
eight manually seeded In/On membership combinations. The integers in the seeded
sets exercise membership logic; the method does not dereference vertex geometry.

Native counts occurrences in the second face's In and On sets separately. One
shared integer present in both sets therefore reaches its two-occurrence threshold;
using a union of distinct shared vertices would change behavior.

Build `tkbo_pave_filler_check_planes.cxx` against the pinned modeling installation,
linking TKBO, TKPrim, TKTopAlgo, TKBRep, TKGeomAlgo, TKGeomBase, TKG3d, TKG2d,
TKMath and TKernel, then compare stdout with the adjacent `.expected` file.
Rust replay: `cargo test -p rustycad-tkbo --test bop_algo_pave_filler_check_planes`.
The uninitialized Rust regression verifies its explicit safe error boundary; it is
not native differential evidence. These supplemental tests do not map additional
legacy declarations or establish complete PerformFF behavior.
