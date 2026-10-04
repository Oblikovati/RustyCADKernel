# BeanFace sampled periodic origins

This probe captures the native `ElCLib::InPeriod` behavior used by localized sample
evaluation. The curve, U, and V samples have nonzero origins, while their raw extremum
parameters are near zero; the expected values therefore distinguish sample origins from
the bean's global bounds. A circle on a torus exercises periodicity in all three axes.

```sh
g++ -std=c++17 -I "$OCCT_INCLUDE" tools/oracle/tkbo_bean_periodic_origins.cxx \
  -L "$OCCT_LIB" -Wl,-rpath,"$OCCT_LIB" \
  -lTKBO -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomAlgo -lTKGeomBase \
  -lTKG2d -lTKG3d -lTKMath -lTKernel -o /tmp/tkbo-bean-periodic-origins
/tmp/tkbo-bean-periodic-origins > tools/oracle/tkbo_bean_periodic_origins.expected
```
