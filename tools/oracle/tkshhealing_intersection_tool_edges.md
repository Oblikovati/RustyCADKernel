# ShapeFix_IntersectionTool edge primitives

This probe uses the pinned OCCT install and directly calls the public
`ShapeFix_IntersectionTool::SplitEdge` and `CutEdge` methods. It records the
split ranges, shared middle vertex, cleared edge flags, endpoint rejection,
ordinary cut result, and the tiny 3D remainder from a trimmed-line p-curve cut.
The last case distinguishes this class's `CutEdge` from the similar
`ShapeFix_SplitTool::CutEdge`: the intersection tool intentionally does not
apply the latter's tiny-range rejection or curve-domain validation.

Build and run with:

```sh
g++ -std=c++17 -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/tkshhealing_intersection_tool_edges.cxx \
  -L"$OCCT_INSTALL/lib" -Wl,-rpath,"$OCCT_INSTALL/lib" \
  -lTKShHealing -lTKTopAlgo -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d \
  -lTKBRep -lTKMath -lTKernel -o /tmp/tksh-intersection-tool-edges
LD_LIBRARY_PATH="$OCCT_INSTALL/lib" /tmp/tksh-intersection-tool-edges
```
