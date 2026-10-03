# ShapeAnalysis_CheckSmallFace::CheckPinFace

The probe creates two equal short circular edges. `CheckPinEdges` accepts the
pair directly, but `CheckPinFace` does not reach that call: native filters the
second edge by `distance <= tolerance` immediately before requiring the same
ratio to exceed 3. The face-level return, map extent, and status remain false/0/OK.

```sh
g++ -std=c++17 -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/shape_analysis_check_pin_face.cxx \
  -L"$OCCT_INSTALL/lib" -Wl,-rpath,"$OCCT_INSTALL/lib" \
  -lTKShHealing -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomBase \
  -lTKG3d -lTKG2d -lTKMath -lTKernel -o /tmp/shape_analysis_check_pin_face
LD_LIBRARY_PATH="$OCCT_BUILD/lin64/gcc/lib:$OCCT_INSTALL/lib" \
  /tmp/shape_analysis_check_pin_face
```
