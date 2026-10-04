# EF candidate tolerance during vertex fusion

Pinned OCCT source is commit `3d097a0328e71b826377d4814ab05ec3c3d23871`, specifically `BOPAlgo_PaveFiller_5.cxx` and `BOPAlgo_Tools.cxx`. The EF branch stores `aTolVnew` in each CoupleOfPaveBlocks; `TreatNewVertices` passes those tolerances to `IntersectVertices`, which expands each pair's accepted distance using the greater of the vertex tolerance and the candidate tolerance. For line/plane intersections, the candidate tolerance also includes half the solver common-part range.

The probe uses three line/plane intersections separated by 0.00041 per coordinate and fuzzy value 0.0004. Adjacent generated points are farther apart than fuzzy alone, but their line/plane common-part tolerances cause OCCT to fuse all three. The exact emitted EF records, split parameters, and fused vertex are in `tkbo_pave_filler_ef_candidate_tolerance.expected`.

Reproduce with the OCCT modeling build:

```sh
OCCT_BUILD=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
g++ -std=c++17 -O0 -g0 -I"$OCCT_BUILD/include/opencascade" \
  tools/oracle/tkbo_pave_filler_ef_candidate_tolerance.cxx \
  -L"$OCCT_BUILD/lin64/gcc/libi" \
  -Wl,-rpath,"$OCCT_BUILD/lin64/gcc/libi" -Wl,--no-as-needed \
  -lTKBO -lTKPrim -lTKTopAlgo -lTKShHealing -lTKGeomAlgo -lTKBRep \
  -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkbo_pave_filler_ef_candidate_tolerance
/tmp/tkbo_pave_filler_ef_candidate_tolerance
```

The Rust regression is `cargo test -p rustycad-tkbo --test bop_algo_pave_filler_ef_multiblock perform_ef_fuses_hits_using_candidate_tolerance_beyond_fuzzy_value`. The evidence covers the line/plane vertex path only; other candidate tolerance formulas and non-analytic solver families remain unproven.
