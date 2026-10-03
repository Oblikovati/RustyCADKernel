# ShapeUpgrade_ClosedFaceDivide

The probe covers a cylindrical face split from an existing seam with the default
and two interior cut points, and the geometric-closure fallback on a sphere.
The near-closed sphere uses `SetPrecision(0.1)` to cover precision propagation
through closure detection and adaptor resolution; its truncated U range does
not qualify at the default confusion tolerance. The probe records the changed
result, resulting face/edge counts, seam count, and ShapeExtend status bits.

```sh
g++ -std=c++17 -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/shape_upgrade_closed_face_divide.cxx \
  -L"$OCCT_INSTALL/lib" -Wl,-rpath,"$OCCT_INSTALL/lib" \
  -lTKShHealing -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomBase -lTKG3d \
  -lTKG2d -lTKMath -lTKernel -o /tmp/shape_upgrade_closed_face_divide
LD_LIBRARY_PATH="$OCCT_BUILD/lin64/gcc/lib:$OCCT_INSTALL/lib" \
  /tmp/shape_upgrade_closed_face_divide
```

Expected output is stored in `shape_upgrade_closed_face_divide.expected`.
