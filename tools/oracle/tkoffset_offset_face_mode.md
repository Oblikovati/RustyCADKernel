# `BRepOffset_Offset(TopoDS_Face, Offset)`

This fixture checks the default outside/arc face constructor on a bounded plane:
status, initial-shape identity, offset face topology and area, validity, and edge
history. OCCT source baseline: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

```sh
g++ -std=c++17 -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/tkoffset_offset_face_mode.cxx \
  -L"$OCCT_INSTALL/lib" -Wl,-rpath,"$OCCT_INSTALL/lib" \
  -lTKOffset -lTKFillet -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomAlgo \
  -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkoffset_offset_face_mode
/tmp/tkoffset_offset_face_mode
```

Only the default offset-outside and arc-join face constructor is delegated to
`BRepOffsetOffsetFace`; the `Created` map overload and other option combinations
remain outside this Rust facade slice.
