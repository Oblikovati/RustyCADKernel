# EF projection with an existing split edge

Pinned OCCT source is commit `3d097a0328e71b826377d4814ab05ec3c3d23871`. The native probe runs `PerformVE` on a point inserted at parameter 0.5 of a vertical edge, then runs `PerformEF` for a plane crossing at parameter 1.0. The crossing belongs to the second of three resulting pave blocks; this verifies `PerformNewVertices` reprojects the generated point, searches all current blocks, and splits the containing one.

Reproduce against the OCCT modeling build:

```sh
OCCT_BUILD=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
g++ -std=c++17 -O0 -g0 -I"$OCCT_BUILD/include/opencascade" \
  tools/oracle/tkbo_pave_filler_ef_multiblock.cxx \
  -L"$OCCT_BUILD/lin64/gcc/libi" \
  -Wl,-rpath,"$OCCT_BUILD/lin64/gcc/libi" -Wl,--no-as-needed \
  -lTKBO -lTKPrim -lTKTopAlgo -lTKShHealing -lTKGeomAlgo -lTKBRep \
  -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkbo_pave_filler_ef_multiblock
/tmp/tkbo_pave_filler_ef_multiblock
```

The Rust replay is `perform_ef_projects_into_the_matching_block_after_ve_split` in `bop_algo_pave_filler_ef_multiblock.rs`. It covers only a straight-edge line/plane crossing after an earlier VE split.
