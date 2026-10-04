# `BRepOffset_Offset::Init` with p-curve-only rails

This fixture covers rails with no 3D curve representation. OCCT adapts each trimmed
p-curve on its located surface, builds the pipe face, then installs the corresponding
U or V isoparametric 3D curve on the source rail. The test retains native's valid
result, edge and vertex counts, source edge tolerances, and surface area.

OCCT source baseline: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

```sh
g++ -std=c++17 -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/tkoffset_offset_rails_pcurve_only.cxx \
  -L"$OCCT_INSTALL/lib" -Wl,-rpath,"$OCCT_INSTALL/lib" \
  -lTKOffset -lTKFillet -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomAlgo \
  -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkoffset_offset_rails_pcurve_only
/tmp/tkoffset_offset_rails_pcurve_only
```

The native output is recorded in `tkoffset_offset_rails_pcurve_only.expected`.
