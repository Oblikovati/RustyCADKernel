# Simple-offset empty vertex samples

Probe source: `tkoffset_simple_offset_empty_vertex_probe.cxx`.

The oracle is OCCT commit `3d097a0328e71b826377d4814ab05ec3c3d23871`. In
`BRepOffset_SimpleOffset::FillVertexData`, lines 335–338 return only when a
vertex has zero incident edges. A standalone free edge has incident edges, but
`FillEdgeData` does not bind offset curves for it; the later centroid divides
by the empty sample count (line 405), then binds the resulting point (lines
421–426).

Build against the pinned installed OCCT libraries:

```sh
g++ -std=c++17 tools/oracle/tkoffset_simple_offset_empty_vertex_probe.cxx \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  -L/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -lTKOffset -lTKTopAlgo -lTKBRep -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkoffset_simple_offset_empty_vertex_probe
LD_LIBRARY_PATH=/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  /tmp/tkoffset_simple_offset_empty_vertex_probe
```

Observed output, one row per endpoint, is `1 1 1 1 0`: `NewPoint` returns
true, all three coordinates are NaN, and tolerance is zero.
