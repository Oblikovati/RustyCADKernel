# ShapeFix_IntersectionTool nonadjacent point crossing

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

These direct native `FixSelfIntersectWire` observations place two intersecting
nonadjacent line edges among two remote edges. They cover an interior point
crossing, a near-endpoint point hit that cuts one edge and splits the other, a
middle-to-endpoint hit that reuses the endpoint vertex, an
endpoint-to-endpoint hit that unions a shared endpoint, a long common segment
that is split and removed, a reversed edge crossing, two point hits between a
circle and line repaired through the repeated scan, endpoint reuse while
removing an overlap segment at each end of an edge, and a short common segment
split on both edges while endpoint vertices are displaced from their curves.
Each row records the return value, split/cut/removed counters, resulting edge
count, and ordered p-curve ranges. They do not establish entire-class parity.

Build and run:

```sh
g++ -std=c++17 -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/tkshhealing_intersection_tool_aggregate.cxx \
  -L"$OCCT_INSTALL/lib" -Wl,-rpath,"$OCCT_INSTALL/lib" \
  -lTKShHealing -lTKTopAlgo -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d \
  -lTKBRep -lTKMath -lTKernel -o /tmp/tksh-intersection-tool-aggregate
LD_LIBRARY_PATH="$OCCT_INSTALL/lib" /tmp/tksh-intersection-tool-aggregate
```
