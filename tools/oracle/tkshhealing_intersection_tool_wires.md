# ShapeFix_IntersectionTool cross-wire point repair

This native probe creates two direct face wires whose first edges cross at an
interior point. It records face reconstruction, per-wire edge counts and
parameter ranges, and confirms the replacement context maps the new face to
itself. The next rows exercise middle-to-endpoint vertex reuse and
endpoint-to-endpoint vertex union. Rust has a separate one-wire guard
regression. The short-overlap case checks an offset intersection that splits
each wire once. Long-overlap cases cover either edge fully containing the
overlap, reversed parameter direction, and one partial interval that retains a
target tail while inserting the source middle segment. The endpoint-cut case
confirms that OCCT trims
the long edge from parameter 0 to 1 but reports no face replacement and leaves
the short edge's range intact.

Build and run against the pinned OCCT install:

```sh
g++ -std=c++17 -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/tkshhealing_intersection_tool_wires.cxx \
  -L"$OCCT_INSTALL/lib" -Wl,-rpath,"$OCCT_INSTALL/lib" \
  -lTKShHealing -lTKTopAlgo -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d \
  -lTKBRep -lTKMath -lTKernel -o /tmp/tksh-intersection-tool-wires
LD_LIBRARY_PATH="$OCCT_INSTALL/lib" /tmp/tksh-intersection-tool-wires
```

Other partial-overlap endpoint arrangements and endpoint cuts that continue
into splitting remain outside this fixture.
