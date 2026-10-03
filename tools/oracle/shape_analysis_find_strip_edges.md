# ShapeAnalysis_CheckSmallFace::FindStripEdges

This probe records edge selection, seam-edge suppression, small-edge skipping,
maximum distance output, and `StatusStrip(DONE3)` for the pinned OCCT method.
The `.expected` file is the exact output from OCCT's shared line-edge fixtures.

Build and run with the repository's pinned OCCT build:

```sh
g++ -std=c++17 -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/shape_analysis_find_strip_edges.cxx \
  -L"$OCCT_INSTALL/lib" -Wl,-rpath,"$OCCT_INSTALL/lib" \
  -lTKShHealing -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomBase \
  -lTKG3d -lTKG2d -lTKMath -lTKernel -o /tmp/shape_analysis_find_strip_edges
LD_LIBRARY_PATH="$OCCT_BUILD/lin64/gcc/lib:$OCCT_INSTALL/lib" \
  /tmp/shape_analysis_find_strip_edges
```
