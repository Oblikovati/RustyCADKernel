# ShapeAnalysis_CheckSmallFace::CheckPin

The native probe checks an elementary support (which is ignored without changing
the output parameters), a smooth pin, and a sharp pin with coincident boundary
poles. The face has a 2x2 Bezier support, matching the pole grids used by the
Rust differential regression. Null-support faces are omitted because native
`CheckPin` dereferences the returned surface handle.

```sh
g++ -std=c++17 -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/shape_analysis_check_pin.cxx \
  -L"$OCCT_INSTALL/lib" -Wl,-rpath,"$OCCT_INSTALL/lib" \
  -lTKShHealing -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomBase \
  -lTKG3d -lTKG2d -lTKMath -lTKernel -o /tmp/shape_analysis_check_pin
LD_LIBRARY_PATH="$OCCT_BUILD/lin64/gcc/lib:$OCCT_INSTALL/lib" \
  /tmp/shape_analysis_check_pin
```
