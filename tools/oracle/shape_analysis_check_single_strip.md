# ShapeAnalysis_CheckSmallFace::CheckSingleStrip

The pinned native fixture exercises two edges sharing endpoint vertices, a
one-edge failure, too many unique vertices, duplicate-first-edge behavior, and
an out-of-tolerance pair. The output includes DONE3 and edge output nullness.

Build and run with the repository's OCCT install:

```sh
g++ -std=c++17 -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/shape_analysis_check_single_strip.cxx \
  -L"$OCCT_INSTALL/lib" -Wl,-rpath,"$OCCT_INSTALL/lib" \
  -lTKShHealing -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomBase \
  -lTKG3d -lTKG2d -lTKMath -lTKernel -o /tmp/shape_analysis_check_single_strip
LD_LIBRARY_PATH="$OCCT_BUILD/lin64/gcc/lib:$OCCT_INSTALL/lib" \
  /tmp/shape_analysis_check_single_strip
```
