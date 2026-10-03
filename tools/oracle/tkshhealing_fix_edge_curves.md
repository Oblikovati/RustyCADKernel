# ShapeFix_Edge curve repair oracle

Run the probe against pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`:

```sh
build="$PWD/target/occt-modeling-full"
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_fix_edge_curves.cxx \
  -L"$lib" -Wl,-rpath,"$lib" \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkshhealing_fix_edge_curves
/tmp/tkshhealing_fix_edge_curves > /tmp/tkshhealing_fix_edge_curves.actual
diff -u tools/oracle/tkshhealing_fix_edge_curves.expected \
  /tmp/tkshhealing_fix_edge_curves.actual
```

The cylinder fixture and each native result are asserted by
`crates/tkshhealing/tests/shape_fix_edge_curves.rs`. Successful curve removal
leaves status OK. Reversal mutates the stored p-curve, adjusts its range, and
clears SameRange/SameParameter when the new range differs from the 3D range.
Missing p-curves cannot support 3D reconstruction and report FAIL1.
