# ShapeAnalysis_CheckSmallFace::CheckStripFace

This probe verifies both branches: first accepting a single two-edge strip and
then falling back to `FindStripEdges` when multiple distinct endpoint vertices
make the single-strip topology check fail. The one-edge fixture also checks
output mutation and status on failure.

Build and run with the pinned OCCT install:

```sh
g++ -std=c++17 -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/shape_analysis_check_strip_face.cxx \
  -L"$OCCT_INSTALL/lib" -Wl,-rpath,"$OCCT_INSTALL/lib" \
  -lTKShHealing -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomBase \
  -lTKG3d -lTKG2d -lTKMath -lTKernel -o /tmp/shape_analysis_check_strip_face
LD_LIBRARY_PATH="$OCCT_BUILD/lin64/gcc/lib:$OCCT_INSTALL/lib" \
  /tmp/shape_analysis_check_strip_face
```
