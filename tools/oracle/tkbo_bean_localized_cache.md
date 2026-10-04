# BeanFace localized cache and premarked ranges

This private native probe seeds the face cache with an exclusion and initializes the
range manager as already intersected before calling `ComputeLocalized`. It records the
resulting interval flags, exclusion reset, and released grid state. A second run uses a
curve interval below the sampling threshold to pin the native full-range behavior. The
probe targets recursive localization rather than public dispatch.

Build and capture with the configured OCCT installation:

```sh
g++ -std=c++17 -I "$OCCT_INCLUDE" tools/oracle/tkbo_bean_localized_cache.cxx \
  -L "$OCCT_LIB" -Wl,-rpath,"$OCCT_LIB" \
  -lTKBO -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomAlgo -lTKGeomBase \
  -lTKG2d -lTKG3d -lTKMath -lTKernel -o /tmp/tkbo-bean-localized-cache
/tmp/tkbo-bean-localized-cache > tools/oracle/tkbo_bean_localized_cache.expected
```
