# BRepOffset_MakeSimpleOffset safe-distance oracle

`tkoffset_simple_safe_distance.cxx` calls the public `GetSafeOffset` member on a
null maker, a box, a single planar face, and a cylinder's lateral face. Its output
fixture is `crates/tkoffset/tests/fixtures/simple_safe_distance.txt` and was
generated with OCCT revision `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The box's sampled maximum normal angle is 90 degrees, so `expectedTolerance = 1`
produces `0.31830985435280207` with the primitive's max vertex tolerance. A single
face and a cylinder's lateral seam produce no non-smooth face pair, leaving the
cached angle at zero. A pair of sewn Bezier extrusion faces records the trimmed
23-step sample result for a curved junction. The resulting IEEE infinity and NaN
are part of OCCT's observable division behavior and are preserved by the Rust helper.

Build and run against the OCCT build used by the port:

```sh
c++ -std=c++17 -I<occt-build>/include/opencascade \
  tools/oracle/tkoffset_simple_safe_distance.cxx \
  -L<occt-build>/lin64/gcc/libi \
  -Wl,-rpath,<occt-build>/lin64/gcc/libi \
  -lTKOffset -lTKPrim -lTKBO -lTKBRep -lTKTopAlgo -lTKGeomAlgo -lTKGeomBase \
  -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o tkoffset_simple_safe_distance
./tkoffset_simple_safe_distance
```
