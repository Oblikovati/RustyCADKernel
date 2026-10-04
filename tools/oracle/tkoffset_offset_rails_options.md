# `BRepOffset_Offset::Init` rail pipe options and Bezier rails

This fixture passes six configurations through the public edge-rail constructor:
`Polynomial`, `Tol`, and `GeomAbs_Shape` are varied independently. The three input
edges carry translated quadratic Bezier curves, so the pipe takes the generic
`GeomFill_Pipe` branch instead of its line special case. It records native status,
face validity, topology, B-spline pole counts/degrees, area, and mutated source edge
tolerances. Enabling polynomial section construction changes the output section degree
and area; the other sampled configurations retain the source behavior for this input.

OCCT source baseline: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

```sh
g++ -std=c++17 -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/tkoffset_offset_rails_options.cxx \
  -L"$OCCT_INSTALL/lib" -Wl,-rpath,"$OCCT_INSTALL/lib" \
  -lTKOffset -lTKFillet -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomAlgo \
  -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkoffset_offset_rails_options
/tmp/tkoffset_offset_rails_options
```

The native output is recorded in `tkoffset_offset_rails_options.expected`.
