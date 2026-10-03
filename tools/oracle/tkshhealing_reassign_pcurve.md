# ShapeBuild_Edge::ReassignPCurve oracle

The probe uses pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871` and records
whether reassignment succeeded, whether p-curves are stored on the old and new
faces, seam state, and p-curve ranges/endpoints. Cases cover an ordinary stored
curve, a plane projection returned by `BRep_Tool` without stored edge data, a
missing source p-curve, and moving one side of a seam onto a face that already
has a p-curve in both FORWARD and REVERSED edge orientations.

```sh
build="$PWD/target/occt-modeling-full"
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_reassign_pcurve.cxx \
  -L"$lib" -Wl,-rpath,"$lib" \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkshhealing_reassign_pcurve
/tmp/tkshhealing_reassign_pcurve > /tmp/tkshhealing_reassign_pcurve.actual
diff -u tools/oracle/tkshhealing_reassign_pcurve.expected \
  /tmp/tkshhealing_reassign_pcurve.actual
```
