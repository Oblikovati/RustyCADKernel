# `BRepOffset_Offset::Init(Path, Edge1, Edge2, Offset, ...)`

This fixture covers the native edge-rail constructor with identity-located 3D line
curves and no optional start/end connector edges. It verifies the trimmed pipe face,
connector topology, shared rail geometry, source status, area, validity, and tolerances.
OCCT source baseline: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

```sh
g++ -std=c++17 -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/tkoffset_offset_rails.cxx \
  -L"$OCCT_INSTALL/lib" -Wl,-rpath,"$OCCT_INSTALL/lib" \
  -lTKOffset -lTKFillet -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomAlgo \
  -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkoffset_offset_rails
/tmp/tkoffset_offset_rails
```

Native output is recorded in `tkoffset_offset_rails.expected`. P-curve-only rails and nonidentity locations are covered by the companion
`tkoffset_offset_rails_pcurve_only` and `tkoffset_offset_rails_located` fixtures.
Degenerated circular rails, closed paths, and supplied `FirstEdge`/`LastEdge` have
separate companion fixtures.
