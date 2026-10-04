# BeanFace localized public Perform

This probe exercises the public `IntTools_BeanFaceIntersector::Perform` path on flat
Bezier and cubic B-spline faces, with identity and translated face locations. It runs
each solver twice to preserve the shared surface-cache reuse case. The B-spline case has
degree 3 and three distinct knots in both directions, so it enters native localization.

Build and capture with the configured OCCT installation:

```sh
g++ -std=c++17 -I "$OCCT_INCLUDE" tools/oracle/tkbo_bean_localized_perform.cxx \
  -L "$OCCT_LIB" -Wl,-rpath,"$OCCT_LIB" \
  -lTKBO -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomAlgo -lTKGeomBase \
  -lTKG2d -lTKG3d -lTKMath -lTKernel -o /tmp/tkbo-bean-localized-perform
/tmp/tkbo-bean-localized-perform > tools/oracle/tkbo_bean_localized_perform.expected
```

The fixture retains 17-digit native endpoints; Rust comparisons use an absolute
tolerance of `1e-12`.
