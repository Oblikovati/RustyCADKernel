# ShapeUpgrade_SplitCurve2dContinuity oracle

The probe was built against OCCT revision `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Rows record status, shared split-sequence identity, the corrected B-spline knot count (or -1
for wrapped/non-spline curves), the resulting split values, and `myNbCurves`. Fixtures include
an exact `Last - PConfusion()` knot boundary and repeated knot removal to retain the 2D source
loop behavior.

```sh
native_build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
lib="$native_build/lin64/gcc/libi"
g++ -std=c++17 -I"$native_build/include/opencascade" \
  tools/oracle/tkshhealing_split_curve2d_continuity.cxx \
  -L"$lib" -Wl,-rpath,"$lib" \
  -lTKShHealing -lTKGeomBase -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkshhealing_split_curve2d_continuity
/tmp/tkshhealing_split_curve2d_continuity
```

Expected output is in `tkshhealing_split_curve2d_continuity.expected`.
