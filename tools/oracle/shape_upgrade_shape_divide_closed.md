# ShapeUpgrade_ShapeDivideClosed

The probe divides a cylindrical lateral face with the default, two-cut, and
three-cut settings. It records whole-shape status and topology, then checks that
the installed `ShapeUpgrade_ClosedFaceDivide` retains the requested count and
has a null wire divider.

```sh
g++ -std=c++17 -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/shape_upgrade_shape_divide_closed.cxx \
  -L"$OCCT_INSTALL/lib" -Wl,-rpath,"$OCCT_INSTALL/lib" \
  -lTKShHealing -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomBase -lTKG3d \
  -lTKG2d -lTKMath -lTKernel -o /tmp/shape_upgrade_shape_divide_closed
LD_LIBRARY_PATH="$OCCT_BUILD/lin64/gcc/lib:$OCCT_INSTALL/lib" \
  /tmp/shape_upgrade_shape_divide_closed
```

Expected output is stored in `shape_upgrade_shape_divide_closed.expected`.
