# ShapeFix_Edge::FixAddPCurve oracle

The probe records return value, HasPCurve/IsSeam, every individual DONE/FAIL
flag, and both stored p-curve types, ranges, and endpoints for OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`. The torus cases exercise
`TranslatePCurve` on both a line and a B-spline. The plane case also records
OCCT's synthesized planar p-curve query result after its documented early
return.

```sh
build="$PWD/target/occt-modeling-full"
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_fix_add_pcurve.cxx \
  -L"$lib" -Wl,-rpath,"$lib" \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkshhealing_fix_add_pcurve
/tmp/tkshhealing_fix_add_pcurve > /tmp/tkshhealing_fix_add_pcurve.actual
diff -u tools/oracle/tkshhealing_fix_add_pcurve.expected \
  /tmp/tkshhealing_fix_add_pcurve.actual
```

Rust replays each row in `crates/tkshhealing/tests/shape_fix_edge_add_pcurve.rs`.
The oracle covers the status and p-curve outcomes of these exact cases; it does
not imply full `ShapeFix_Edge` coverage.
