# ShapeAnalysis_CheckSmallFace::CheckTwisted

The probe compares an elementary surface, a flat Bezier surface, and a folded
Bezier surface whose sampled support normals reverse. It records found status,
output parameters, and `StatusTwisted`.

```sh
g++ -std=c++17 -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/shape_analysis_check_twisted.cxx \
  -L"$OCCT_INSTALL/lib" -Wl,-rpath,"$OCCT_INSTALL/lib" \
  -lTKShHealing -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomBase \
  -lTKG3d -lTKG2d -lTKMath -lTKernel -o /tmp/shape_analysis_check_twisted
LD_LIBRARY_PATH="$OCCT_BUILD/lin64/gcc/lib:$OCCT_INSTALL/lib" \
  /tmp/shape_analysis_check_twisted
```
