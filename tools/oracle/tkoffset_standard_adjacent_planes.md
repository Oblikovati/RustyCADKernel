# Adjacent planar face offset baseline

This fixture extracts one shared edge and its two faces from a `4 x 5 x 6` box,
then builds an open shell from those faces with their solid orientations. The pinned
OCCT `BRepOffset_MakeOffset` skin/Arc run creates two offset planes and the curved
shared-edge patch. It confirms a valid, three-face shell with one generated face for
the shared edge, edge images for free boundary edges, and vertex images at both ends.

The analysis reports one convex interval for the shared edge. The native builder
creates its patch with `BRepOffset_Offset(edge, edgeImage1, edgeImage2, offset)`;
the source offset-edge image map has no binding for that shared edge while
`Generated(sharedEdge)` returns the connector face.

OCCT source baseline: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

```sh
occt=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
lib="$occt/lin64/gcc/libi"
g++ -std=c++17 -I"$occt/include/opencascade" \
  tools/oracle/tkoffset_standard_adjacent_planes.cxx \
  -L"$lib" -Wl,-rpath,"$lib" -lTKOffset -lTKPrim -lTKTopAlgo -lTKBool \
  -lTKBRep -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkoffset_adjacent_planes
/tmp/tkoffset_adjacent_planes
```

The captured output is `tkoffset_standard_adjacent_planes.expected`.
