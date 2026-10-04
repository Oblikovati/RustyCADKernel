# Standard offset maker state probe

This fixture verifies native `BRepOffset_MakeOffset` state that is defined after
default construction, `Initialize`, `AddFace`, and `Clear`. It records the initial
shape, result nullness, completion state, join type, closing-face deduplication,
and bad-shape nullness without invoking the offset algorithm.

It was captured from pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`:

```sh
c++ -std=c++17 -I<occt-build>/include/opencascade \
  tools/oracle/tkoffset_standard_maker_state.cxx \
  -L<occt-build>/lin64/gcc/libi -Wl,-rpath,<occt-build>/lin64/gcc/libi \
  -lTKOffset -lTKFeat -lTKFillet -lTKBool -lTKBO -lTKPrim -lTKShHealing \
  -lTKTopAlgo -lTKGeomAlgo -lTKGeomBase -lTKBRep -lTKG3d -lTKG2d -lTKMath \
  -lTKernel -o /tmp/tkoffset-standard-maker-state
/tmp/tkoffset-standard-maker-state
```

The Rust state regression is `cargo test -p rustycad-tkoffset --test
standard_offset_state`. It verifies stored initializer arguments and state resets;
it does not claim standard offset execution.
