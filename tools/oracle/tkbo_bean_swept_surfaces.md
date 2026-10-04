# BeanFace offset and swept surface families

This native `IntTools_BeanFaceIntersector::Perform` probe intersects an edge with a plane and sphere offset surface, a toroidal surface represented as a surface of revolution, and a circular linear extrusion. Each case runs twice. The expected records preserve all result ranges, including the four narrow/touching ranges OCCT reports for the revolution case.

Build against the pinned OCCT include and library directories:

```sh
g++ -std=c++17 -I "$OCCT_INCLUDE" tools/oracle/tkbo_bean_swept_surfaces.cxx \\
  -L "$OCCT_LIB" -Wl,-rpath,"$OCCT_LIB" \\
  -lTKBO -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomAlgo -lTKGeomBase \\
  -lTKG2d -lTKG3d -lTKMath -lTKernel -o /tmp/tkbo-bean-swept-surfaces
/tmp/tkbo-bean-swept-surfaces > tools/oracle/tkbo_bean_swept_surfaces.expected
```

Rust compares every endpoint against the captured native value with absolute tolerance `1e-12`.
