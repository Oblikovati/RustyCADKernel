# BRepOffset simple offset callback oracle

This probe observes the pinned OCCT `BRepOffset_SimpleOffset` callbacks and then feeds the
modification through `BRepTools_Modifier`. It uses the same bounded plane fixture as the
Rust integration tests. The output file records the native values, including false-callback
sentinels and orientation-sensitive lookups.

Build and run from the repository root after the pinned OCCT modeling build exists:

```sh
build="$PWD/target/occt-modeling-full"
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/tkoffset_simple_offset_callbacks_probe.cxx \
  -L"$lib" -Wl,-rpath,"$lib" \
  -lTKOffset -lTKPrim -lTKBO -lTKTopAlgo -lTKBRep -lTKGeomAlgo \
  -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/occt_simple_offset_callbacks_probe
/tmp/occt_simple_offset_callbacks_probe
```

`vertex-orientation` and the `vertex-point`/`vertex-query` lines intentionally show the
orientation of the vertex keys selected by OCCT traversal: the forward query misses while
its reversed form matches the prepared map. The face and edge reversed queries miss. False
NewSurface/NewCurve/NewPoint callbacks preserve all sentinel outputs.
