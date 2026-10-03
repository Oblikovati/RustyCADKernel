# ShapeAnalysis_CheckSmallFace::CheckStripEdges

This probe exercises the pinned OCCT `CheckStripEdges` call on coincident,
within-tolerance, beyond-tolerance, exactly-at-tolerance, and edge-derived
tolerance line pairs. It records both the boolean return and `dmax`, including
the early-return output value.

Build against the pinned OCCT install:

```sh
g++ -std=c++17 -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/shape_analysis_check_strip_edges.cxx \
  -L"$OCCT_INSTALL/lib" -Wl,-rpath,"$OCCT_INSTALL/lib" \
  -lTKShHealing -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomBase \
  -lTKG3d -lTKG2d -lTKMath -lTKernel -o /tmp/shape_analysis_check_strip_edges
LD_LIBRARY_PATH="$OCCT_BUILD/lin64/gcc/lib:$OCCT_INSTALL/lib" \
  /tmp/shape_analysis_check_strip_edges
```

The expected output is stored in the adjacent `.expected` file.
