# `ShapeFix_Wire::FixLacking` bend follow-ups

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

This two-edge planar fixture has a 0.001 p-curve gap, high edge tolerances, and a
shared 3D vertex. It drives the successful bend branch, then the two
`FixSelfIntersectingEdge` calls and `FixIntersectingEdges` call that OCCT runs
after bending. The output records the final flags, shared vertex tolerance, and
samples on each adjusted p-curve. Rust checks all of these observations in
`shape_fix_wire_lacking::successful_curve_bend_runs_both_native_follow_up_repairs`.

Build and compare with the pinned toolkit:

```sh
g++ -std=c++17 -I/path/to/occt-install/include/opencascade \
  tools/oracle/tkshhealing_wire_lacking_bend.cxx \
  -L/path/to/occt-install/lib -Wl,-rpath,/path/to/occt-install/lib \
  -lTKShHealing -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d -lTKBRep -lTKMath -lTKernel \
  -o /tmp/tkshhealing-wire-lacking-bend
LD_LIBRARY_PATH=/path/to/occt-install/lib /tmp/tkshhealing-wire-lacking-bend
```

The fixture establishes this bend branch only; topology-changing bend cases and
all other `ShapeFix_Wire` branches require their own native observations.
