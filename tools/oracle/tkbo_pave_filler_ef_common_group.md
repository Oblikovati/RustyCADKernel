# EF face membership on an existing multi-block common block

Pinned OCCT source is commit `3d097a0328e71b826377d4814ab05ec3c3d23871`. The probe calls `PerformVF`, `PerformEE`, then `PerformEF` on two coincident edges lying on the same bounded plane face. This creates one common block shared by both pave blocks before EF; EF must append the face once while retaining both block members.

Reproduce against the OCCT modeling build:

```sh
OCCT_BUILD=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
g++ -std=c++17 -O0 -g0 -I"$OCCT_BUILD/include/opencascade" \
  tools/oracle/tkbo_pave_filler_ef_common_group.cxx \
  -L"$OCCT_BUILD/lin64/gcc/libi" \
  -Wl,-rpath,"$OCCT_BUILD/lin64/gcc/libi" -Wl,--no-as-needed \
  -lTKBO -lTKPrim -lTKTopAlgo -lTKShHealing -lTKGeomAlgo -lTKBRep \
  -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkbo_pave_filler_ef_common_group
/tmp/tkbo_pave_filler_ef_common_group
```

The Rust replay is the `perform_ef_reuses_common_block_shared_by_coincident_edges` test in `bop_algo_pave_filler_ef_multiblock.rs`. This fixture proves this two-edge common-block case only; it does not establish arbitrary multi-block merging or tolerance recomputation for all common-block shapes.
