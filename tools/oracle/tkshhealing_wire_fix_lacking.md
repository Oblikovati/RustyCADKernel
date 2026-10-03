# ShapeFix_Wire missing-connection oracle

`tkshhealing_wire_fix_lacking.cxx` compares the pinned single-edge and aggregate
`FixLacking` overloads with the Rust regression in `shape_fix_wire_lacking.rs`.
The `closed=` row is a closed-edge insertion case; it does not exercise OCCT's
`doAddLong` neighbor replacement or its `Context()` updates. Those remain
unverified by this fixture. The bent-p-curve path also omits OCCT's subsequent
`FixSelfIntersectingEdge` and `FixIntersectingEdges` calls in the Rust port, so
the Rust `FixLacking` overloads are not complete.

Build against the pinned OCCT modeling tree:

```sh
NATIVE_ROOT=/path/to/RustyCADKernel/target/occt-modeling-full
g++ -std=c++17 -I"$NATIVE_ROOT/include/opencascade" tools/oracle/tkshhealing_wire_fix_lacking.cxx \
  -L"$NATIVE_ROOT/lin64/gcc/libi" -Wl,-rpath,"$NATIVE_ROOT/lin64/gcc/libi" \
  -lTKShHealing -lTKBRep -lTKTopAlgo -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkshhealing_wire_fix_lacking
/tmp/tkshhealing_wire_fix_lacking
```
