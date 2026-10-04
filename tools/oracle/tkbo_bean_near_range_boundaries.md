# BeanFace near-range boundary seeding

This probe calls native private `ComputeNearRangeBoundaries` directly using
access-only visibility. A line segment from `(0,0,1e-7)` to `(1,0,1e-7)` lies
within the `2e-7` combined tolerance of a bounded plane face. Starting from one
unclassified range `[0,1]`, endpoint distance checks and range walks mark the
whole interval with flag 2, preserving zero-width unmarked endpoint intervals.

The test initializes `IntTools_Context` explicitly because this private stage
normally runs through `Perform`, which creates the context first. The probe does
not change OCCT behavior. The Rust regression calls the isolated stage helper
with matching adaptor bounds, tolerances, context and range manager.

Build against the pinned OCCT output:

```sh
g++ -std=c++17 -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkbo_bean_near_range_boundaries.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,target/occt-modeling-full/lin64/gcc/libi \
  -lTKBO -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomAlgo -lTKGeomBase \
  -lTKG2d -lTKG3d -lTKMath -lTKernel \
  -o /tmp/tkbo-bean-near-range-boundaries
/tmp/tkbo-bean-near-range-boundaries
```
