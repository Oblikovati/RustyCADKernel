# ShapeFix_Edge parameter and vertex repair oracle

The probe uses pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
It records return values, all individual DONE/FAIL flags, the combined DONE flag,
SameParameter/SameRange flags, and edge/vertex tolerances. Rust replays every
recorded scalar in `shape_fix_edge_parameters.rs`.

```sh
build="$PWD/target/occt-modeling-full"
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_fix_edge_parameters.cxx \
  -L"$lib" -Wl,-rpath,"$lib" \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkshhealing_fix_edge_parameters
/tmp/tkshhealing_fix_edge_parameters > /tmp/tkshhealing_fix_edge_parameters.actual
diff -u tools/oracle/tkshhealing_fix_edge_parameters.expected \
  /tmp/tkshhealing_fix_edge_parameters.actual
```

The gap cases deliberately retain native status resetting: vertex repair may
clear earlier DONE flags despite changing tolerances. Context-based vertex
repair preserves the original vertices and records replacements instead.
This probe does not establish coverage of the entire ShapeFix_Edge class.
