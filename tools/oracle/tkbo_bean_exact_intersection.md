# BeanFace exact intersection range seeding

This probe calls the native private `ComputeAroundExactIntersection` directly via
access-only visibility in the probe translation unit. The input is a unit sphere's
lateral face and a line edge from `(-2,0,0)` to `(2,0,0)`, with an initial unmarked
range `[-2,2]`. OCCT's exact HInter finds the two sphere crossings. The helper walks
outward from both hits and marks the two zero-distance neighborhoods with flag 2,
leaving the central separating interval and outer intervals unmarked. The minimum
squared distance becomes zero.

The access macro changes visibility only in this probe; no OCCT source or algorithm
is modified. The Rust regression invokes its isolated exact-stage helper with the
same adaptors, tolerances, bounds, and initial range.

Build against the pinned OCCT output:

```sh
g++ -std=c++17 -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkbo_bean_exact_intersection.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,target/occt-modeling-full/lin64/gcc/libi \
  -lTKBO -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomAlgo -lTKGeomBase \
  -lTKG2d -lTKG3d -lTKMath -lTKernel \
  -o /tmp/tkbo-bean-exact-intersection
/tmp/tkbo-bean-exact-intersection
```
