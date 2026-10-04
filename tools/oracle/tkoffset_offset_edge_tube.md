# `BRepOffset_Offset::Init(TopoDS_Edge, Offset)`

This covers the rolling-ball free-boundary edge tube on an identity-located 3D
line. It compares positive and negative offsets, status, initial-shape identity,
face orientation, topology, area, tolerance, validity, and the native `Generated`
edge branch. OCCT source baseline: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

```sh
g++ -std=c++17 -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/tkoffset_offset_edge_tube.cxx \
  -L"$OCCT_INSTALL/lib" -Wl,-rpath,"$OCCT_INSTALL/lib" \
  -lTKOffset -lTKFillet -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomAlgo \
  -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkoffset_offset_edge_tube
/tmp/tkoffset_offset_edge_tube 1
/tmp/tkoffset_offset_edge_tube -1
```

P-curve-only and nonidentity-located source edges remain unported. OCCT exposes
this as an `Init` overload after default construction, not a two-argument C++
constructor.
