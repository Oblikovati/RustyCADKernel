# IntTools_BeanFaceIntersector localized ranges

This probe calls the pinned private `ComputeLocalized()` implementation directly for a
flat Bezier face and a cubic B-spline face. Each face is crossed by the same vertical edge;
the B-spline has degree 3 and three distinct knots in each direction, so both cases enter
the native localized path. The output records every marked interval and range flag.

The source is OCCT revision `3d097a0328e71b826377d4814ab05ec3c3d23871`. Rebuild against
the configured OCCT headers and libraries with:

```sh
g++ -std=c++17 -I "$OCCT_INCLUDE" tools/oracle/tkbo_bean_localized.cxx \
  -L "$OCCT_LIB" -Wl,-rpath,"$OCCT_LIB" \
  -lTKBO -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomAlgo -lTKGeomBase \
  -lTKG2d -lTKG3d -lTKMath -lTKernel -o /tmp/tkbo-bean-localized
/tmp/tkbo-bean-localized > tools/oracle/tkbo_bean_localized.expected
```

`$OCCT_INCLUDE` is the configured `include/opencascade` directory and `$OCCT_LIB` is the
matching `lin64/gcc/libi` directory. `private` is exposed only in this standalone probe
because `ComputeLocalized()` is private in the pinned header.
