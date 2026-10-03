# ShapeFix_Wire::Perform oracle

`tkshhealing_fix_wire_perform.cxx` probes the pinned OCCT `ShapeFix_Wire::Perform`
implementation for an unloaded fixer and two free-wire insertion orders. The
fixture intentionally uses no face so it validates the aggregate path that can
run without the still-unported face repair stages.

Build against the repository's configured OCCT tree:

```sh
OCCT=target/occt-modeling-full
g++ -std=c++17 -I"$OCCT/include/opencascade" \
  tools/oracle/tkshhealing_fix_wire_perform.cxx \
  -L"$OCCT/lin64/gcc/libi" -Wl,-rpath,"$OCCT/lin64/gcc/libi" \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkshhealing-fix-wire-perform
/tmp/tkshhealing-fix-wire-perform
```

The checked-in `.expected` file records the native return value and aggregate
status queries. It does not assert face-ready behavior.
