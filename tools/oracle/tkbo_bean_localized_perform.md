# BeanFace localized public Perform

This probe exercises the public `IntTools_BeanFaceIntersector::Perform` path on flat and genuinely curved Bezier and cubic B-spline faces. Each surface is tested at identity and translated face locations, and each solver runs twice to exercise shared surface-cache reuse. Curved patches have noncoplanar poles; a vertical test edge crosses them above the base plane. The cubic B-spline surfaces have three distinct knots in both directions, so they enter native localization.

Build and capture with the configured OCCT installation:

```sh
g++ -std=c++17 -I "$OCCT_INCLUDE" tools/oracle/tkbo_bean_localized_perform.cxx \\
  -L "$OCCT_LIB" -Wl,-rpath,"$OCCT_LIB" \\
  -lTKBO -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomAlgo -lTKGeomBase \\
  -lTKG2d -lTKG3d -lTKMath -lTKernel -o /tmp/tkbo-bean-localized-perform
/tmp/tkbo-bean-localized-perform > tools/oracle/tkbo_bean_localized_perform.expected
```

The fixture retains 17-digit native endpoints; Rust comparisons use an absolute tolerance of `1e-12`.
