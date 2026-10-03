# `ShapeFix_Wire::FixLacking` long-edge topology branch

Pinned OCCT source: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The two adjacent edges share a topological vertex, while their 3D and p-curve
endpoint ranges leave a unit connector gap. With topology mode enabled and
geometry mode disabled, native `FixLacking` copies and replaces both neighboring
edges, inserts the connector from `x=0` to `x=1`, and reports DONE2. The test also
checks the original edge identities are recorded in the reshape context.

Build and run the fixture with the installed OCCT toolkit:

```sh
g++ -std=c++17 -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  tools/oracle/tkshhealing_wire_lacking_long_edge.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -lTKShHealing -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d -lTKBRep -lTKMath -lTKernel \
  -o /tmp/tkshhealing_wire_lacking_long_edge
LD_LIBRARY_PATH=/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  /tmp/tkshhealing_wire_lacking_long_edge
```

This establishes the long-edge insertion branch and its context replacements; it
does not establish the remaining `FixLacking` or `ShapeFix_Wire` branches.
