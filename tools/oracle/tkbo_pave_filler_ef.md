# `BOPAlgo_PaveFiller::PerformEF`

Pinned OCCT source: `BOPAlgo_PaveFiller_5.cxx` at `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The derived probe exposes protected `Init` and `PerformEF` for one straight edge crossing a bounded plane face. It records the generated vertex, EF common-part record, split edge blocks, and face `VerticesIn` classification.

Pinned OCCT worktree is clean at the recorded commit. The five probes cover an interior line/plane crossing, endpoint contact, coplanar overlap, repeat calls in three destructive/fuzzy modes, and three hits fused to a shared vertex. They do not establish non-linear curve/surface coverage or the remaining post-treatment paths.

Reproduce all native snapshots with the modeling build used by adjacent TKBO probes:

```sh
for name in tkbo_pave_filler_ef tkbo_pave_filler_ef_endpoint \
  tkbo_pave_filler_ef_coplanar tkbo_pave_filler_ef_repeat \
  tkbo_pave_filler_ef_fused tkbo_pave_filler_ef_near \
  tkbo_pave_filler_ef_bezier; do
  g++ -std=c++17 -O0 -g0 -I"$OCCT_BUILD/include/opencascade" \
    "tools/oracle/$name.cxx" -L"$OCCT_BUILD/lin64/gcc/libi" \
    -Wl,-rpath,"$OCCT_BUILD/lin64/gcc/libi" -Wl,--no-as-needed \
    -lTKBO -lTKPrim -lTKTopAlgo -lTKShHealing -lTKGeomAlgo \
    -lTKBRep -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
    -o "/tmp/$name"
  "/tmp/$name" > "/tmp/$name.actual"
  diff -u "tools/oracle/$name.expected" "/tmp/$name.actual"
done
```

Rust differential tests: `cargo test -p rustycad-tkbo --test bop_algo_pave_filler_ef`. The near-hit fixture uses points at Y = 0, 0.0003, and 0.0006 with fuzzy 0.0004; OCCT transitively groups the endpoints even though their separation exceeds fuzzy, and the test checks center, tolerance, block ordering, and shared indices. The Bezier probe records native success (`InterfEF` contains the crossing); Rust intentionally returns `Standard_NotImplemented` because BeanFace's finite-domain surface localization is not ported. That test verifies the missing behavior stays explicit instead of being converted into a successful warning-only `PerformEF` call.

This slice still lacks native post-treatment for `PerformCommonBlocks`, `UpdateVerticesOfCB`, `PerformNewVertices`, and complete endpoint/vertex/edge common-part branches. Error/alert ordering, user-break handling, distant transforms, and other curve/surface pairs remain unimplemented or unverified; passing fixtures do not claim full `PerformEF` parity.

Additional pinned probes cover endpoint contact, coplanar edge overlap, repeat calls in destructive/non-destructive and fuzzy/zero-fuzzy modes, and three edge crossings fused to one shared vertex. The latter checks all three EF records, each split block's shared vertex index, face classification, and the generated vertex coordinates/tolerance. Source cases remain limited to straight line edges against one bounded plane face.
