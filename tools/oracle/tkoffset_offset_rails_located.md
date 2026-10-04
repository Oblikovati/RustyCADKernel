# `BRepOffset_Offset::Init` with located source edges

This fixture translates the path and both 3D rails by one nonidentity `TopLoc_Location`.
OCCT reads each edge's curve and composed location, transforms the trimmed curves into
world coordinates for pipe construction, and returns an identity-located face. The
Rust regression compares native status, validity, edge/curve counts, and area.

OCCT source baseline: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

```sh
g++ -std=c++17 -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/tkoffset_offset_rails_located.cxx \
  -L"$OCCT_INSTALL/lib" -Wl,-rpath,"$OCCT_INSTALL/lib" \
  -lTKOffset -lTKFillet -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomAlgo \
  -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkoffset_offset_rails_located
/tmp/tkoffset_offset_rails_located
```

The native output is recorded in `tkoffset_offset_rails_located.expected`.
