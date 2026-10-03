# ShapeFix_Wire degenerated oracle

`tkshhealing_wire_degenerated.cxx` probes the pinned OCCT degenerated-edge fixer behavior. Its output is checked against `tkshhealing_wire_degenerated.expected` and the Rust regressions in `shape_fix_wire_degenerated.rs`.

Build against the pinned OCCT modeling tree:

```sh
NATIVE_ROOT=/path/to/RustyCADKernel/target/occt-modeling-full
g++ -std=c++17 -I"$NATIVE_ROOT/include/opencascade" tools/oracle/tkshhealing_wire_degenerated.cxx \
  -L"$NATIVE_ROOT/lin64/gcc/libi" -Wl,-rpath,"$NATIVE_ROOT/lin64/gcc/libi" \
  -lTKShHealing -lTKBRep -lTKTopAlgo -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkshhealing_wire_degenerated
```
