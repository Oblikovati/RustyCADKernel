# ShapeFix_IntersectionTool cross-wire point repair

This native probe creates two direct face wires whose first edges cross at an
interior point. It records face reconstruction, per-wire edge counts and
parameter ranges, and confirms the replacement context maps the new face to
itself. The next rows exercise middle-to-endpoint vertex reuse and
endpoint-to-endpoint vertex union. Rust has a separate one-wire guard
regression. The final case exercises splitting a short, slightly offset
overlap into one additional edge per wire.

Build and run against the pinned OCCT install:

```sh
g++ -std=c++17 -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/tkshhealing_intersection_tool_wires.cxx \
  -L"$OCCT_INSTALL/lib" -Wl,-rpath,"$OCCT_INSTALL/lib" \
  -lTKShHealing -lTKTopAlgo -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d \
  -lTKBRep -lTKMath -lTKernel -o /tmp/tksh-intersection-tool-wires
LD_LIBRARY_PATH="$OCCT_INSTALL/lib" /tmp/tksh-intersection-tool-wires
```

Long overlap reconstruction and endpoint-cut branches are not covered by this
fixture yet.
