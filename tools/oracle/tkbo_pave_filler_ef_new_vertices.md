# EF generated-vertex projection

Pinned OCCT source is commit `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The probe exposes protected `Init` and `PerformEF`, then records EF rows, generated vertex coordinates and tolerance, and the three resulting pave blocks. This isolates the post-treatment called by native `PerformNewVertices`: it fuses the three EF hits, projects the fused vertex onto each source edge through `IntersectVE(..., false)`, and only then splits the edges. The vertices are spaced so adjacent hits fuse with fuzzy value `0.0004`.

Build and replay against the OCCT modeling libraries:

```sh
OCCT_BUILD=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
g++ -std=c++17 -O0 -g0 -I"$OCCT_BUILD/include/opencascade" \
  tools/oracle/tkbo_pave_filler_ef_new_vertices.cxx \
  -L"$OCCT_BUILD/lin64/gcc/libi" \
  -Wl,-rpath,"$OCCT_BUILD/lin64/gcc/libi" -Wl,--no-as-needed \
  -lTKBO -lTKPrim -lTKTopAlgo -lTKShHealing -lTKGeomAlgo -lTKBRep \
  -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkbo_pave_filler_ef_new_vertices
/tmp/tkbo_pave_filler_ef_new_vertices
```

The native output is checked in `tkbo_pave_filler_ef_new_vertices.expected`; the Rust replay is `cargo test -p rustycad-tkbo --test bop_algo_pave_filler_ef_new_vertices`. It proves only this straight-edge / bounded-plane generated-vertex path. Other EF solver domains, endpoint cases, multi-block grouping, warnings, and cancellation remain outside this fixture's evidence.
