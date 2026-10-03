# ShapeFix_Wire self-intersection aggregate

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The first two rows use the same four line edges with 3D curves and face p-curves as the
direct `ShapeFix_IntersectionTool` point-crossing fixture, with an explicit
`ShapeBuild_ReShape` context. The first disables the single-edge and adjacent
passes to isolate nonadjacent handling. The second keeps native default modes
and exercises the aggregate pass order. A third row uses the Bezier loop
fixture to exercise `ModifyRemoveLoopMode=1`, including the per-edge rescan
and `FixClosed` call. Each row contains the aggregate return, edge count,
removed-segment flag, all 19 native status queries in enum order, every
resulting edge range, and the edge count after applying the context to the
original wire. These are focused cases, not a claim of whole-class parity.

Build and run:

```sh
build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_wire_self_intersection_aggregate.cxx \
  -L"$build/lin64/gcc/libi" -Wl,-rpath,"$build/lin64/gcc/libi" \
  -lTKShHealing -lTKTopAlgo -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d \
  -lTKBRep -lTKMath -lTKernel -o /tmp/tksh-wire-self-agg
LD_LIBRARY_PATH="$build/lin64/gcc/libi" /tmp/tksh-wire-self-agg
```
