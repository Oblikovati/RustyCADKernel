# ShapeAnalysis_CheckSmallFace::CheckSplittingVertices

The probe checks a vertex within its tolerance of an edge interior, a remote
vertex, and a face with no vertices. It records found split count, map extents,
status, and output compound child count.

```sh
g++ -std=c++17 -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/shape_analysis_check_splitting_vertices.cxx \
  -L"$OCCT_INSTALL/lib" -Wl,-rpath,"$OCCT_INSTALL/lib" \
  -lTKShHealing -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomBase \
  -lTKG3d -lTKG2d -lTKMath -lTKernel -o /tmp/shape_analysis_check_splitting_vertices
LD_LIBRARY_PATH="$OCCT_BUILD/lin64/gcc/lib:$OCCT_INSTALL/lib" \
  /tmp/shape_analysis_check_splitting_vertices
```
