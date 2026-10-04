# `BRepOffset_MakeOffset::CheckInputData`

This probe records native offset validation behavior for the null global offset, a
nonzero face override, and disconnected face components. The implementation is
pinned to OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`; run it against the
matching modeling build and compare stdout with
`tkoffset_check_input_data.expected.txt`.

```sh
c++ -std=c++17 \
  -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkoffset_check_input_data.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,$PWD/target/occt-modeling-full/lin64/gcc/libi \
  -lTKOffset -lTKFeat -lTKFillet -lTKBool -lTKBO -lTKPrim \
  -lTKShHealing -lTKTopAlgo -lTKGeomAlgo -lTKGeomBase -lTKBRep \
  -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkoffset-check-input-data
/tmp/tkoffset-check-input-data | diff -u tools/oracle/tkoffset_check_input_data.expected.txt -
```

The Rust method currently validates connectivity, C0 continuity, the 21-by-21
normal grid, and face vertex parameters. Native progress interruption and
`BuildFaceComp` selection from face-specific offsets still need implementation
before the method can be considered a complete port.
