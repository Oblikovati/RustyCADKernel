# Adjacent crossing edge tolerance with reshape context

Pinned OCCT source: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The fixture uses two crossing p-curves and matching 3D line curves. The second
curve's nearest crossing segment has a 0.45 local deviation from the line joining
the crossing to the shared endpoint. With a reshape context, native
`ShapeFix_Wire::FixIntersectingEdges` copies both edges and their endpoint
vertices before raising the copied edge tolerance. The original shared vertex
stays at tolerance `1e-7`; the applied shared vertex and second edge have
tolerance `0.45000045`. Both DONE1 and DONE6 are set.

The adjacent repair assertion is in `wire_intersection_fix_tests.rs`. Rebuild and
run the native fixture with:

```sh
g++ -std=c++17 -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  tools/oracle/tkshhealing_wire_context_edge_tolerance.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -lTKShHealing -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d -lTKBRep -lTKMath -lTKernel \
  -o /tmp/wire_context_edge_tolerance
LD_LIBRARY_PATH=/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  /tmp/wire_context_edge_tolerance
```

This adds evidence for the context edge-tolerance branch only; it does not establish
full `ShapeFix_Wire` or toolkit parity.
