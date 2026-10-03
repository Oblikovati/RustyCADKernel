# ShapeAnalysis_CheckSmallFace::CheckPinEdges

The native probe compares matched circular arcs whose first/second derivatives
agree at the shared endpoint, and collinear line edges whose zero second
derivatives cause the native angle exception path to return false. It records
the exact boolean for each case.

```sh
g++ -std=c++17 -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/shape_analysis_check_pin_edges.cxx \
  -L"$OCCT_INSTALL/lib" -Wl,-rpath,"$OCCT_INSTALL/lib" \
  -lTKShHealing -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomBase \
  -lTKG3d -lTKG2d -lTKMath -lTKernel -o /tmp/shape_analysis_check_pin_edges
LD_LIBRARY_PATH="$OCCT_BUILD/lin64/gcc/lib:$OCCT_INSTALL/lib" \
  /tmp/shape_analysis_check_pin_edges
```
