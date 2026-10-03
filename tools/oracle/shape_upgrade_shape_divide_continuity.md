# ShapeUpgrade_ShapeDivideContinuity

The probe builds the same degree-two B-spline wire with a C0 knot for each run.
It compares C0/C1/C2 boundary criteria and a loose C1 tolerance on that wire,
then applies C0/C1/C2 surface criteria to a tensor-product B-spline face. It
also applies C1 pcurve splitting to a plane face with one C0 boundary pcurve,
both at the native `PConfusion` default and at a loose 2D tolerance. It records
whole-shape result counts and ShapeExtend flags.

```sh
g++ -std=c++17 -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/shape_upgrade_shape_divide_continuity.cxx \
  -L"$OCCT_INSTALL/lib" -Wl,-rpath,"$OCCT_INSTALL/lib" \
  -lTKShHealing -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomBase -lTKG3d \
  -lTKG2d -lTKMath -lTKernel -o /tmp/shape_upgrade_shape_divide_continuity
LD_LIBRARY_PATH="$OCCT_BUILD/lin64/gcc/lib:$OCCT_INSTALL/lib" \
  /tmp/shape_upgrade_shape_divide_continuity
```

Expected output is stored in `shape_upgrade_shape_divide_continuity.expected`.
