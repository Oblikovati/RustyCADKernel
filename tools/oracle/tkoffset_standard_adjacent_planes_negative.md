# Negative offset for adjacent planar faces

Pinned OCCT source baseline: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`BRepOffset_MakeOffset.cxx` SHA-256:
`002411457684d23891e07f651394e6d3701d40aa8a1f6577f6505aec5c048688`.
This fixture offsets two connected planar faces by `-0.5` with arc joins.
OCCT classifies their shared edge as convex (`TypeOfConcavity` value `1`),
while its negative-offset branch requests concave edges (value `0`) for the
arc patch. OCCT nevertheless completes by intersecting and trimming the two
offset plane boundaries. The native output in
`tkoffset_standard_adjacent_planes_negative.expected` records a valid shell
with two faces, eight edges, area `48`, both face images, and no generated
shared-edge join.

The active Rust regression asserts `Standard_NotImplemented` until the
boundary-intersection and trim path is ported. This is an explicit gap check,
not a passing parity claim. A local diagnostic that skipped the unmatched arc
join reported success but produced areas `30 + 24`, rather than OCCT's `21 +
27`; that behavior was rejected. Do not remove the error guard or enable
successful output until the trimmed boundary geometry, topology, and history
match the native fixture.

Reproduce against the pinned native modeling build:

```sh
occt=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
lib="$occt/lin64/gcc/libi"
g++ -std=c++17 -I"$occt/include/opencascade" \
  tools/oracle/tkoffset_standard_adjacent_planes_negative.cxx \
  -L"$lib" -Wl,-rpath,"$lib" -lTKOffset -lTKPrim -lTKTopAlgo -lTKBool \
  -lTKBRep -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkoffset-standard-adjacent-planes-negative
/tmp/tkoffset-standard-adjacent-planes-negative
```
