# BeanFace extremum range seeding

This probe calls native private `ComputeUsingExtremum` with access-only visibility.
The first case is a line coincident with the supporting plane but crossing a narrow
trim; it exercises the parallel-extremum bisection and endpoint range walk. The
second case is a circle crossing a plane twice and exercises nonparallel extrema.
Both cases initialize the range manager and context explicitly, as `Perform` does
before this stage.

Build against the pinned OCCT output:

```sh
g++ -std=c++17 -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkbo_bean_using_extremum.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,target/occt-modeling-full/lin64/gcc/libi \
  -lTKBO -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomAlgo -lTKGeomBase \
  -lTKG2d -lTKG3d -lTKMath -lTKernel \
  -o /tmp/tkbo-bean-using-extremum
/tmp/tkbo-bean-using-extremum
```

`tkbo_bean_using_extremum.expected` records the native ranges and flags. The Rust
unit fixtures invoke the same stage directly and compare each range boundary.
