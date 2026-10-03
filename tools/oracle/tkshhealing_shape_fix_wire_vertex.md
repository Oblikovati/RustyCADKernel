# `ShapeFix_WireVertex` oracle

Pinned source: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`,
`src/ModelingAlgorithms/TKShHealing/ShapeFix/ShapeFix_WireVertex.cxx`.

Build from the repository root:

```sh
build=target/occt-modeling-full
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_shape_fix_wire_vertex.cxx \
  -L"$lib" -Wl,-rpath,"$lib" \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkshhealing-shape-fix-wire-vertex
/tmp/tkshhealing-shape-fix-wire-vertex
```

`*.expected` records counts and endpoint identity/point/tolerance/parameter observations.
`FixSame` adds the selected shared vertex but retains old endpoint vertices; `Fix` first
removes endpoints and then reconnects them. The tested status-4 record also preserves the
upstream `ShapeAnalysis_WireVertex::SetStart` following-slot behavior.
